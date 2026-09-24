---
baseline_commit: 0fbb6ce
model: claude-opus-5-5  # session default; 11.2's creation ran on claude-opus-5 -- recorded so the ledger row is readable
---

# Story 11.3: Night Falls, Day Breaks

Status: in-progress

## Story

As the boss,
I want a sun by day and a moon by night, on a slow clock, with the game booting at night,
so that the valley is a place where time passes, and the night I judged everything under is still
the night I see when I start.

## Not stacked — branch off `main`

`main` is `0fbb6ce`, clean; the crates are byte-identical to `53ba44e`, which is what
`target/debug/{gui,simd}` were built from (`gui --version` → `gui build 53ba44e`). Branch slug
`story-11-3-night-falls-day-breaks`. Every "X did not change" AC is proved with
`git diff 0fbb6ce..HEAD`, never against a branch tip.

## Rulings taken at creation (Wolf, 2026-09-24)

1. **Day length: `TICKS_PER_DAY = 24_000`** (1,000 ticks per hour). Normal (10 ticks/s) turns a day
   in 40 min, Fast (50 ticks/s, the daemon's maximum) in 8 min.
2. **The moon moves.** It travels an arc and passes **exactly** through the approved key direction
   (`SUN_AZIMUTH_DEGREES = 40.0398`, `SUN_ELEVATION_DEGREES = 17.66`, `atmosphere.rs:35,39`) at the
   boot hour. **Every capture is pinned** — to `--clock <h>` when given, else to the boot hour —
   so no capture ever follows the tick, and every existing pixel guard reads unchanged without
   being edited (AC2, AC7).
3. **The day is judged against an artifact Wolf approves BEFORE the day table is written**
   (UX-DR22 opening half). The story builds the objective mechanism first against a **PROVISIONAL**
   `day_lighting()` (the creation probe's p3 values, marked `// PROVISIONAL — not approved`),
   renders candidate day frames headless and **STOPS** for Wolf's pick (Task 4). Tests read
   `day_lighting()` by name, never its literals, so they survive the pick unchanged; the literal
   pin and the day-value mutation rows land only after it. **This story runs in two sittings:
   Tasks 0-4a, STOP, then 4b onward.** It is one story, not a split.
4. **Sky: table-driven, flat, `Exposure` fixed at 10.5.** The clock drives `ClearColor`, the star
   and aurora fade, and the fog/rim colour together from the light tables. Bevy's procedural
   `Atmosphere` was probed live at creation (below) and **not adopted** — Wolf: *"I think we start
   with 1 but might change my mind.. none of those athmosphere shots are convincing execpt maybe p1
   is okeish"*. So nothing here may foreclose it: no sky abstraction, and the sky colour stays ONE
   value per frame that a later change can replace. No follow-up story is filed.

## What creation already measured

**The control, on `53ba44e` (= `0fbb6ce`'s crates), fresh `simd --pause-at 120` per capture:**

```
capture range check: warm-lit pixels=23433 ground-median-luminance=69 near-white-area=0.3906%
  blown-pool=0.3637% p99-luminance=153.5 resolution=1280x720        (control a, exit 0)
control b: same line, and `cmp control-a.png control-b.png` → IDENTICAL (same-build floor = 0)
RED --lights-off sun: warm-lit 27043, ground median 50 → panics capture.rs:1531 (floor 55), exit 101
```

So a frozen boot capture is **bit-reproducible**, and the range check **sees the key light** — the
exact thing this story's clock drives. AC2 leans on both facts.

**The creation probe** — a throwaway patch in an isolated `git worktree` (the main tree was never
edited), env-var overrides on `night_lighting()`, `sun_direction()` and an optional `Atmosphere`
on the camera; boot framing, all effects on, fresh `simd --pause-at 120` each. The probe with no
override set was **`cmp`-identical to the control**, so every difference below is the override's.
Frames: `11-3-signoff/probe-*.png`, side by side in `probe-contact-sheet.png`.

| frame | key el / lux / RGB | ambient / RGB | sky | ground median | near-white |
| --- | --- | --- | --- | ---: | ---: |
| control (approved night) | 17.66° / 7,000 / (178,200,240) | 1,500 / (108,128,170) | (5,12,28) | 69 | 0.3906 % |
| p1 night + `Atmosphere` | same | same | procedural | 64 | 0.3855 % |
| **p3 day, flat** | 40° / 12,000 / (255,244,228) | 4,000 / (190,210,235) | (110,155,205) | **161** | 0.8519 % |
| p4 day + `Atmosphere` | same as p3 | same as p3 | procedural | 155 | 0.6581 % |
| p5 sunset + `Atmosphere` | 3° / 6,000 / (255,170,110) | 2,000 / (150,150,190) | procedural | 68 | 0.3896 % |
| p7 sunset, flat | same as p5 | same as p5 | (190,110,80) | 73 | 0.3921 % |

What it established:

- **`Atmosphere` loads and renders under lavapipe with no warning** — but the moon, at 7,000 lux,
  lights it as a sun (the shader sums every directional light), so at night it washes the sky
  grey-brown and dims the aurora (p1); by day it reads beige at this scale and EV (p4). Not adopted.
- **A day reads as day on the flat sky (p3)**: ground median 69 → 161 — and it still PASSES every
  night band (161 ≤ 180; near-white 0.8519 % ≤ 0.946 %, a thin 0.09 pp margin). So AC8 keeps the
  band at every hour; only a day Wolf approves that actually trips it earns a ruling.
- **`sharpness.py` on the frames** (Rec.601): `sky-stars` `lap_mean` is 1.8812 on the all-effects
  control (11.2's 2.27 predates the haze) and **0.5228** on p3 — a 72 % fall.
- **No moon disc appears at boot framing.** The key's source is in front of the camera but above
  the top of the frame (`atmosphere.rs:407-413`); a sphere on its bearing did not show (p2, not
  filed). A visible moon is not in this story.
- **p3's table is the starting point for Task 4's candidates**, not an approved look.

## Acceptance Criteria

1. **Time of day is client-side and derives from the wire tick alone** (NFR5's carve-out, made
   checkable — a mechanism AC on purpose: the architectural edge IS the requirement).
   `hour = (BOOT_HOUR + (tick % 24_000) as f32 / 1_000.0) mod 24` — FLOAT division, so the hour
   moves every tick, not in whole-hour steps — from `Mirror::tick()` (`client-core/src/lib.rs:111`),
   `BOOT_HOUR = 22.0`, `TICKS_PER_DAY = 24_000`. `git diff 0fbb6ce..HEAD --stat` on
   `crates/protocol`, `crates/sim-core`, `crates/client-core` and `crates/simd` is empty. "Boots at
   night" therefore means **daemon tick 0**: a client joining a long-running daemon sees that
   daemon's hour, which is correct under NFR5 and is how the vehicle card must be read.
2. **Boot is the approved night, byte for byte.** The creation recipe's capture — with no
   `--clock`, and again with `--clock 22` — is **`cmp`-identical** to the control above
   (`53ba44e`), and its range-check line is the control's line plus `clock=`. Every existing
   rendered guard in `crates/gui/tests/pixel_guard.rs` passes **unedited**, and no constant in that
   file or in `capture.rs` changes. If byte-identity cannot be reached, the dev stops and reports
   the diff to Wolf — it is not re-baselined.
3. **The key light follows the clock and never lights from below the horizon.** Sampled every
   0.01 h over 24 h: whenever the key's illuminance is > 0 its forward vector points downward
   (`forward.y < 0`); at 22.0 it is exactly `sun_direction()` with the night table's colour and
   illuminance (the moon); at 12.0 it carries the approved day table's colour and illuminance (the
   sun), higher than the moon's approved elevation.
4. **No pop.** Over the same 0.01 h sweep, no step changes any driven value (sky RGB, ambient RGB and
   brightness, key illuminance, star and aurora fade) by more than 2 % of that value's night-to-day
   range (a value whose night and day entries are equal must not move at all). Key RGB and
   direction are exempt **only while its illuminance is 0** — that is where the sun/moon swap
   happens, and a swap there is invisible by construction. Dusk and dawn are ramps, not cuts.
5. **The sky follows the clock, and moves as one.** Stars and aurora are fully present at 22.0 and
   fully gone at 12.0; `ClearColor`, `DistanceFog.color` and the rim dissolve's target colour are
   the **same** colour at every sampled hour (`tech-art-guidelines.md:291-293`: "these three
   colours move together or not at all"). `Exposure` is `10.5` at every hour.
6. **`--clock <hour>` pins the clock for the whole run** and reaches the live key light and sky (a
   test through `configured_app_with_snapshot`, the shape of
   `lights_steady_reaches_the_live_flicker_system`, `ingest.rs:3051`). It takes `0 ≤ h < 24`; a
   malformed or out-of-range value errors naming the range. Without `--clock` the hour advances
   with the wire tick (a test drives two snapshots 1,000 ticks apart and reads one hour's
   difference).
7. **Every capture is pinned, and says to what.** `--capture` without `--clock` pins the boot hour
   (the epic's "a capture that does not pin the clock is not evidence", made the default rather
   than an error, so no existing recipe or guard breaks). The range-check line prints the hour
   actually RENDERED (`current_hour()`, not the pin's value) as `clock=<h> (--clock)` or
   `clock=22.00 (capture default)`. A test drives a capture app 1,000 ticks on and reads the hour
   unmoved. The format change breaks the literal in `mutations/9-1-*.sh:54` — re-point that row
   and show it still KILLS.
8. **Each pinned hour is range-checked.** The existing band (`capture.rs:1527-1558`) applies at
   every hour, unchanged — the creation probe's day frame passes it (ground 161 ≤ 180, near-white
   0.8519 % ≤ 0.946 %). If the day Wolf approves in Task 4 trips a band, the dev records both
   figures and **Wolf rules** (a per-hour skip in the `band_applies` shape, `capture.rs:1519-1525`,
   is the option to put to him); **no constant is raised** and nothing is skipped without that
   ruling.
9. **Night turns into day, rendered.** A new `#[ignore]`d guard in `pixel_guard.rs` captures
   `--clock 22` and `--clock 12` (one fresh `Daemon::spawn` each, `--static-world`, the harness's
   existing shape) and asserts: the noon ground median exceeds the night's by more than 10
   (creation probe: 69 → 161), and the `sky-stars` window's `rec601_lap_mean` falls by at least
   50 % (creation probe: 1.8812 → 0.5228, −72 %). Four captures at each hour are taken for the
   record with each hour's floor stated, and the bars are re-read against the APPROVED day before
   the guard is committed.
10. **F8 switches the key light whatever body it is, and switching it back restores the CLOCK's
    value.** At `--clock 12`, F8 off → illuminance 0; F8 on → the day table's illuminance, not
    `night_lighting()`'s 7,000. `--lights-off sun` and `ambient` hold at every hour.
11. **The haze follows the ambient it is derived from.** `VolumetricFog.ambient_color` and
    `ambient_intensity` (`0.1 × ambient_brightness / 80`, `ingest.rs:2069-2076`) track the clock's
    ambient; at 22.0 they equal today's values exactly. A test at `--clock 12` presses F4 off then
    on and reads the DAY ambient back: `apply_effect` (`ingest.rs:1714`) re-inserts
    `volumetric_fog()` and is the second writer here.
12. **Opening artifact (UX-DR22).** Before the PROVISIONAL day table is replaced or pinned, ≥ 3
    candidate day frames (`--clock 12`) and one dusk frame (`--clock 17.5`) are filed under
    `11-3-signoff/` with their tables, and Wolf picks one. The approved frame is committed as `11-3-signoff/approved-day-<sha>.png`, and the
    table that produced it is what ships.
13. **The vehicle.** Wolf watches one full cycle at Fast and signs off both halves: the night
    against `10-8-signoff/approved-moonlit-camp-3479a43-a.png`, the day against AC12's approved
    frame. At the same sitting he judges AO in the day frame with F7 (**#106**'s routed look
    judgement) and reads NFR6 at noon fullscreen as in 11.2's AC13; each ruling is recorded.
14. Every mutation row in `mutations/11-3-night-falls-day-breaks.sh` is shown to KILL, the table's
    output pasted into the Dev Agent Record.

## Tasks / Subtasks

- [x] **Task 0 — confirm the control on YOUR build** (AC2)
  - [x] Build, check `gui --version` names your HEAD, take two controls with the creation recipe
        (no `--clock` yet — it does not exist). Both must match the creation line above and `cmp`
        identical to each other. File them as `11-3-signoff/control-<sha>-{a,b}.png`.
- [x] **Task 1 — the clock** (AC1, AC6, AC7)
  - [x] NEW `crates/gui/src/clock.rs`: `TICKS_PER_DAY`, `TICKS_PER_HOUR`, `BOOT_HOUR`,
        `pub fn hour_at(tick: u64) -> f32`, and `pub struct ClockPin(pub Option<f32>)` (resource).
        One `pub fn current_hour(mirror, pin) -> f32` is the ONLY place the hour is computed.
  - [x] `--clock <hour>` in `parse_args_from` (`ingest.rs:1054-1273`): `Args` field, branch and
        range check. `ClockPin` is `Some(h)` from `--clock`, else `Some(BOOT_HOUR)` when
        `--capture` is given, else `None` (follow the tick). Insert it where `--lights-steady` is
        inserted (`:667-669`).
  - [x] `app.init_resource::<ClockPin>()` in `projection_systems` beside `LightingToggles`
        (`ingest.rs:754`), and register the clock-reading systems there too: `tests/capture.rs` and
        `tests/headless.rs` build apps from `projection_systems` alone and would panic with
        "Resource does not exist" (the comment at `ingest.rs:745-751` records that exact defect).
  - [x] Unit tests: `hour_at(0) == BOOT_HOUR`, `hour_at(500) == BOOT_HOUR + 0.5`,
        `hour_at(1_000) == BOOT_HOUR + 1`, wraps at 24;
        parse accepts `0`, `12`, `23.99`, rejects `24`, `-1`, `x`; a capture without `--clock` is
        pinned at `BOOT_HOUR`; a seat run without `--clock` is not.
- [x] **Task 2 — the key light follows the clock** (AC3, AC4, AC10)
  - [x] Keep ONE key `DirectionalLight` (the `SunLight` entity, `ingest.rs:1532-1545`): it is the
        moon while the sun is below the horizon and the sun while above. Two shadowed directional
        lights would double the cascade cost and `VolumetricLight` needs a shadow map on each.
  - [x] `pub fn key_at(hour) -> (Vec3 direction, Color, f32 illuminance)` in `atmosphere.rs`.
        Symmetric arcs: sun above the horizon 06:00–18:00, moon 18:00–06:00, each body's
        illuminance ramping to 0 at its horizon, so the swap happens where both are dark. The
        moon's arc must return **exactly** `sun_direction()` at `BOOT_HOUR` — see the trap list.
        (Reviewed at creation: with the moon's azimuth sweeping 180° over its 12 h, 15°/h, it rises
        at az −19.96°, peaks ~20.4° at midnight and sets at 160.04° — a sane low winter moon.)
  - [x] Route every write through `apply_lighting_toggles` (`ingest.rs:1785-1831`): it rewrites
        illuminance and brightness EVERY FRAME from `night_lighting()`, so a separate clock system
        writing the same fields loses every frame. Make it read the clock's values instead; the
        key's `Transform` gets one writer beside it.
  - [x] Convert `the_approved_sun_lights_downward` (`atmosphere.rs:416-424`) and
        `the_installed_sun_entity_aims_downward_onto_the_valley` (`ingest.rs:3673-3701`) to
        pinned-time guards at `BOOT_HOUR`, and add the 0.01 h sweep (AC3 below-horizon invariant,
        AC4 continuity). `APPROVED_DOWNWARD_FLOOR` stays hand-written and unchanged.
  - [x] The F8 test at `--clock 12` (AC10) — extend the toggle test at `ingest.rs:3840-3940`.
- [x] **Task 3 — the sky follows the clock** (AC5, AC11)
  - [x] `ClearColor` (a resource, inserted at `ingest.rs:627`) is written from the clock's sky
        colour; `Exposure` is not touched. Comment on **#113** that 11.3 settled it as "the sky is
        authored per hour, not exposed" (Wolf's ruling 4), and leave closing it to Wolf.
  - [x] Stars: the one shared opaque material (`atmosphere.rs:267-272`) lerps its `base_color`
        from `star` toward the current sky colour — no blend mode change, so the night draw is
        untouched. Aurora: multiply its `base_color` alpha (already `AlphaMode::Blend`,
        `:275-282`) from 1 to 0. Store both handles in a resource; today they are stored nowhere.
  - [x] Rim: `ProjectionAssets.terrain` (`project.rs:430-436`) holds one handle per
        (slot, rim level); rewrite their `base_color` with `rim_dissolved_color` against the
        current sky, ONLY when the sky colour changed (`Changed`/compare), never per frame blind.
  - [x] `DistanceFog.color`, `ClearColor` and the rim target come from ONE value per frame (AC5's
        equality test reads all three off the live app).
  - [x] `volumetric_fog()`'s ambient follows the clock's ambient through the same derivation (AC11).
- [ ] **Task 4 — the opening artifact, and the STOP** (AC12)
  - [ ] **4a.** Render ≥ 3 candidate day tables at `--clock 12` and one dusk at `--clock 17.5`
        (the sun still ~5° up — at 18.25 the key has ramped dark and the frame is ambient-only;
        tell Wolf that the key goes dark AT the horizon, so the warm light is the last hour before
        it). Each candidate is a temporary edit of the PROVISIONAL `day_lighting()` literals in an
        isolated `git worktree` — never in the shared tree ([[probe-sabotage-leaks-into-wolfs-build]])
        — boot framing, all effects on, fresh `simd --pause-at 120` each. File each frame with
        its table and range-check line under `11-3-signoff/`. Start from p3 (above).
  - [ ] **STOP. Hand the frames to Wolf.** Record his words verbatim in the Dev Agent Record.
        The session resumes at 4b.
  - [ ] **4b.** Write the picked table into `day_lighting()`, drop the PROVISIONAL marker, and pin
        its literals in `appearance_tables_pin_the_cold_boot_palette` (`appearance.rs:330`) beside
        the night literals. `night_lighting()`'s literals do not change.
- [ ] **Task 5 — the instrument, and its test** (AC8, AC9)
  - [ ] The instrument is the capture at a pinned hour:
        `gui <port> --headless --static-world --lights-steady --subdiv 4 --frames 160 --clock <h> --capture <png>`,
        read by its own range-check line (now carrying `clock=`) and by `11-2-signoff/sharpness.py`.
  - [ ] Its test is AC9's guard. Run the RED in Verification **before** accepting its green.
  - [ ] Run every existing rendered guard UNEDITED (`cargo test -p gui --test pixel_guard -- --ignored`)
        — the capture default pin is what keeps them valid (AC2).
- [ ] **Task 6 — docs** (AC5)
  - [ ] `docs/tech-art-guidelines.md`: a day row beside the night rows in the Lights table
        (`:48-78`), the clock constants, and the sky-follows-the-clock rule; strike
        "The boot frame is a night scene" (`:160`) into "boots at 22:00"; mark #113's line (`:215`)
        with how 11.3 settled it.
  - [ ] `README.md`: `--clock` in the flag table (`:238-252`) with the capture default pin, and
        the seat's Fast route (`tui` + `+`) under "At the vehicle".
- [ ] **Task 7 — the sitting** (AC13)
  - [ ] Write `11-3-signoff/vehicle-card.md` in the seat's form (below). Questions for Wolf: the
        night vs the approved moonlit camp; the day vs AC12's frame; the dusk; AO at noon (#106);
        NFR6 at noon fullscreen.
- [ ] **Task 8 — mutations and the gate** (AC14)
  - [ ] `mutations/11-3-night-falls-day-breaks.sh`, at minimum: the hour ignores the tick; `--clock`
        parsed but not applied; the key's direction not driven; a below-horizon key left lit (drop
        the horizon ramp); `apply_lighting_toggles` restores `night_lighting()` at noon; stars not
        faded; the rim target left on the night sky; `DistanceFog.color` left on the night sky;
        `BOOT_HOUR` moved; a capture without `--clock` follows the tick; the hour steps in whole
        hours (integer division); F4-on restores the night haze ambient at noon.
  - [ ] Every row `assert s.count(old) == 1`. Run `audit-mutations.py` **after** `cargo fmt`.

## Dev Notes

### Scope guardrails — do NOT

- Do NOT touch `protocol`, `sim-core`, `client-core` or `simd` (AC1). The tick is already on the
  mirror; nothing about time goes on the wire.
- Do NOT change `night_lighting()`'s literals, `SUN_AZIMUTH_DEGREES`, `SUN_ELEVATION_DEGREES`,
  `Exposure { ev100: 10.5 }`, a `BOOT_*` constant, the point-light table, flicker, or any capture
  constant (`capture.rs:559-635`, `pixel_guard.rs`). The night is approved; this story adds a day
  beside it.
- Do NOT add a key. F1–F12 are spent (#118) and the gui has no speed control — that is 8.3's
  scope ("Master of Time"). The seat reaches Fast through `tui` on the same daemon; the seat pins
  an hour with `--clock` at launch.
- Do NOT dim torches, lanterns or the campfire by day, and do NOT change their emissives. Whether
  they should is a question for the Task 4 frames, not a default.
- Do NOT touch DoF, SSAO, Bloom, FXAA or the haze's density, ramp or volume (11.1/11.2, ruled).
  #120 (haze corner arcs) and #121 (prepass) stay open and out of scope.
- Do NOT rebuild the enclosed-sky oracle (#108) — a day sky makes a colour-keyed sky mask even
  less viable; say so on #108 if Task 4 shows it.
- Do NOT add a `bevy` feature or a dependency.

### What already exists — build on it

- `Mirror::tick()` (`client-core/src/lib.rs:111`) via `Res<MirrorResource>` (`ingest.rs:341`) —
  the tick is readable from any main-world system today. `TickClock` (`blend.rs:10`) is the
  interpolation clock; do not reuse it for time of day.
- `night_lighting()` (`appearance.rs:40-50`) — the ONE table every night value reads. Its type
  `NightLighting` becomes `LightTable` (type rename only; the fn keeps its name).
- `apply_lighting_toggles` (`ingest.rs:1785-1831`, `.after(ProjectionSet)` `:755-759`) — already
  the per-frame writer of every light intensity and the emissives.
- The `band_applies` skip in `validate_capture_ranges_with_report` (`capture.rs:1519-1525`) — the
  shape to offer Wolf if the approved day trips a band (AC8); not built by default.
- `rec601_lap_mean` (`pixel_guard.rs:95`) and `sharpness.py`'s `sky-stars` window (60,10 → 460,110).

### Key decisions & traps

- **Byte-identity at the boot hour is arithmetic, not luck.** The moon's direction at `BOOT_HOUR`
  must come out of the SAME float expression as `sun_direction()`. Write the arc as offsets that
  are exactly zero/one there — e.g. `azimuth = SUN_AZIMUTH_DEGREES + rate * (h - BOOT_HOUR)` and
  `elevation = SUN_ELEVATION_DEGREES * (f(h) / f(BOOT_HOUR))` (`a / a == 1.0` exactly in IEEE) —
  then feed the same `sun_direction` maths. A formula that merely lands "within 1e-6" moves the
  shadow cascades and breaks AC2's `cmp`. The same holds for EVERY driven value at `BOOT_HOUR`:
  the night↔day blend weight must be exactly `0.0` across the whole night (a clamped window, not a
  smooth curve that lands at 1e-8); the moon's horizon ramp must saturate at exactly `1.0` well
  below 17.66°; and colours are mixed only through a path that is exact at factor 0 (Bevy's
  `Srgba::mix`, `bevy_color-0.19.0/src/srgba.rs:262-272`).
- **`apply_lighting_toggles` is the second writer.** Anything the clock writes to a light at spawn
  or in its own system is overwritten from `night_lighting()` the same frame
  ([[spawn-is-not-the-only-writer]]). AC10 exists to catch exactly this.
- **Every existing guard captures at tick ~120 and assumed night.** The harness spawns `simd 0`
  (`pixel_guard.rs:152-191`) and the gui's `--static-world` requests the tick-120 freeze itself.
  Following the tick they would render hour 22.12 — the moon 1.8° further along at 15°/h — and
  the equality controls (`CONTROL_OPEN_SNOW_LL_MEDIAN == 91`, `pixel_guard.rs:274`) would go red on
  a correct clock. The capture default pin (AC7) is
  what prevents that; do not touch the constants.
- **The all-lights-off guard (`pixel_guard.rs:829`) is NOT `--static-world`** and lands on an
  unpinned tick ≥ 100 — the reason the default pin must be the hour, not "whatever tick the
  capture fires on".
- **The atmosphere at night is lit by the moon as if it were a sun.** Bevy's atmosphere shader sums
  every directional light (`bevy_pbr-0.19.0/src/atmosphere/functions.wgsl:216-254`) and the night
  key is 7,000 lux — see the probe section for what that looks like.
- **`ClearColor`, stars and aurora bypass `Exposure`** (#113; `pbr.wgsl:81-84` lit path only;
  `bevy_render-0.19.0/src/view/mod.rs:1238-1251` clear colour written raw). Changing `Exposure` by
  day would move the land and leave the sky — do not reach for it.
- **Near-white and the ground ceiling are NIGHT calibrations.** Day snow in sunlight is near-white
  by physics. AC8's skip is how a day capture is honest, not a ceiling raise.
- **One fresh `simd --pause-at 120` per capture**; `--frames 160`; a mutant binary outlives a source
  restore — rebuild and check `gui --version` shows no `-dirty` before any capture.
- **`scripts/gate.sh` has no thread knob** and its Bevy apps exhaust 23 GB — run it as
  `RUST_TEST_THREADS=2 scripts/gate.sh`, foreground.

### Project Structure

| File | Change |
| --- | --- |
| `crates/gui/src/clock.rs` | NEW — the hour, `ClockPin`, `current_hour` |
| `crates/gui/src/lib.rs` | UPDATE — `mod clock` |
| `crates/gui/src/appearance.rs` | UPDATE — `LightTable` rename, `day_lighting()` (after Task 4), blend helper |
| `crates/gui/src/atmosphere.rs` | UPDATE — `key_at(hour)`, star/aurora handles and fade, pinned-time guards |
| `crates/gui/src/ingest.rs` | UPDATE — `--clock`, `apply_lighting_toggles` reads the clock, key transform writer, sky/fog/`ClearColor`, haze ambient |
| `crates/gui/src/project.rs` | UPDATE — rim materials re-tinted when the sky changes |
| `crates/gui/src/capture.rs` | UPDATE — `clock=` (the rendered hour) in the range-check line; band unchanged |
| `_bmad-output/implementation-artifacts/mutations/9-1-*.sh` | UPDATE — re-point the range-check literal (`:54`) |
| `crates/gui/tests/pixel_guard.rs` | UPDATE — AC9's night-to-day guard only; existing guards unedited |
| `docs/tech-art-guidelines.md`, `README.md` | UPDATE — Task 6 |
| `_bmad-output/implementation-artifacts/11-3-signoff/` | NEW — controls, Task 4 candidates, approved day, vehicle card |
| `_bmad-output/implementation-artifacts/mutations/11-3-night-falls-day-breaks.sh` | NEW |

### Verification

From the repo root. Full gate: `RUST_TEST_THREADS=2 scripts/gate.sh` (foreground, ~1,700 s). Then
`scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/11-3-night-falls-day-breaks.sh`
and `git status --porcelain` (#104).

**The control, which RAN at creation** — one fresh daemon per capture:

```bash
./target/debug/simd 7531 --pause-at 120 &
./target/debug/gui 7531 --headless --static-world --lights-steady --subdiv 4 --frames 160 \
  --capture _bmad-output/implementation-artifacts/11-3-signoff/control-<sha>-a.png
# creation, exit 0, twice, cmp-identical:
#   warm-lit pixels=23433 ground-median-luminance=69 near-white-area=0.3906% blown-pool=0.3637% p99=153.5
# creation RED, --lights-off sun: ground-median 50, panic capture.rs:1531, exit 101
```

**Once `--clock` exists (the obligation the dev inherits):**

```bash
# NIGHT — must be cmp-identical to the control (AC2):
./target/debug/gui <port> ... --clock 22 --capture night-<sha>.png
# NOON — the range-check line must print "clock=12.00 (--clock)", and the ground median must
#        exceed night's 69 by more than 10 (AC9; creation probe p3 read 161):
./target/debug/gui <port> ... --clock 12 --capture noon-<sha>.png
python3 _bmad-output/implementation-artifacts/11-2-signoff/sharpness.py night-<sha>.png noon-<sha>.png
#   sky-stars lap_mean: night 1.8812 (all-effects control), noon at least 50% lower (p3: 0.5228)
```

**THE DELIBERATE RED (AC9's guard):** make `current_hour` ignore `ClockPin` (return the tick-derived
hour). **EXPECTED:** the noon capture renders hour ~22.1 and is a night frame; the range-check
line prints `clock=22.1x` despite `--clock 12` (it prints the RENDERED hour, AC7), and AC9's guard
goes RED on the ground-median clause. A line that printed the pin's value would read `clock=12`
over a night frame — which is why AC7 prints the rendered hour. **RESTORE:** revert, rebuild, check
`gui --version` shows no `-dirty`, re-capture.

**A second RED, for AC10:** make `apply_lighting_toggles` write `night_lighting().directional_illuminance`
again. **EXPECTED:** the F8 test at `--clock 12` fails reading 7,000 where the day table's value is
owed.

**At the vehicle (Task 7's card writes it this way):**

```powershell
# The hour comes from the daemon's tick, so "boots at night" means a FRESH daemon (tick 0 = 22:00).
# WSL, shell 1 — restart it before judging the boot night
simd 7451
# PowerShell, from D:\Workspace\frostvein — look at the boot night FIRST
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4')
# WSL, shell 2 — then fast-forward; the daemon's speed is global. One + is normal -> fast:
# a full day is 8 min, and dawn arrives ~2.7 min in (22:00 -> 06:00 = 8,000 ticks at 50/s)
tui 7451
# judging frames, pinned:
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--clock','22')
.\scripts\launch-gui.ps1 -GuiArgs @('--subdiv','4','--clock','12')
```

### References

- `_bmad-output/planning-artifacts/epics.md:2259-2291` (Story 11.3), `:2112-2148` (Epic 11 rules),
  `:98` (NFR5 carve-out), `:218` (UX-DR22)
- `10-8-lighting-and-atmosphere-re-judged-under-the-sun.md:107-128` — Wolf ruled the key a NIGHT
  key ("Night and moonlight"); `10-8-signoff/approved-moonlit-camp-3479a43-{a,b}.png`
- `11-2-the-miniature.md` — `sharpness.py`, the one-fresh-daemon rule, `apply_effect`, the haze
  derivation; `11-2-signoff/sharpness.py`
- Issues: **#113** (sky bypasses Exposure — routed here), **#106** (AO look — routed here),
  **#118** (no free keys), **#120**, **#121**, **#108** (out of scope, named above)
- Bevy 0.19.0: `bevy_light/src/directional_light.rs:73-145` (fields; cascades rebuilt every frame
  regardless, `cascade.rs:195-254`, so moving the key costs nothing extra);
  `bevy_light/src/ambient_light.rs:6-37` (per-camera `AmbientLight`, cd/m²);
  `bevy_light/src/lib.rs:130-157` (`light_consts::lux`); `bevy_pbr/src/fog.rs:55-75`
  (`DistanceFog`, incl. `directional_light_color`)

### Previous-story intelligence

- 11.2 made frozen captures **deterministic** (`b5a0e2a`): same-build floors are 0, so a `cmp` is a
  legitimate AC2 oracle here, not a lucky draw.
- 11.2's vehicle card used devpod commands and three instruments failed at the seat; write the card
  in the `launch-gui.ps1` form above from the start.
- Author as `Völundr <jeicei75@gmail.com>` on the **first** commit (`--author` every time; this
  clone defaults to `jeicei75`); no `Co-Authored-By`/`Claude-Session` trailers (CLAUDE.md §8). Wolf
  merges mid-session — check `git merge-base --is-ancestor origin/main HEAD` before every push.

## Dev Agent Record

### Agent Model Used

gpt-6-sol (high)

### Debug Log References

- Task 0: `cargo build --offline -p gui -p simd` passed; `./target/debug/gui --version` printed `gui build fc3dd08` without `-dirty`.
- Fresh daemons on ports 7531 and 7532 each paused at tick 120. Both captures printed `warm-lit pixels=23433 ground-median-luminance=69 near-white-area=0.3906% blown-pool=0.3637% p99-luminance=153.5 resolution=1280x720`. `cmp` of the two PNGs exited 0.
- Task 1 RED: `tick_clock_moves_fractionally_and_wraps`: `left: 22.0 right: 22.5` (hour ignored tick). `clock_flag_accepts_hours_and_rejects_out_of_range_values`: `called Result::unwrap() on an Err value: invalid port`. `clock_pin_reaches_the_live_app_and_capture_defaults_to_boot`: `left: None right: Some(None)`. `an_unpinned_seat_follows_two_wire_snapshots_one_hour_apart`: `left: 0.0 right: 1.0`. `capture_clock_note_names_the_rendered_hour_and_source`: `left: "" right: " clock=12.00 (--clock)"`. All subsequently green. Mutation rows and results will be recorded after the committed table runs.
- Task 2 RED: `key_arc_uses_the_approved_boot_direction_and_the_provisional_noon_table`: `assertion failed: noon_direction.y < night_direction.y`. `lit_key_never_points_up_or_jumps_in_illuminance`: `both keys are dark at dawn; left: 7000.0 right: 0.0`. `clock_drives_the_installed_key_direction_color_and_illuminance`: installed `Vec3(0.7295178, -0.3033679, 0.61300206)` vs noon `Vec3(0.5864819, -0.6427876, 0.49281135)`. `f8_restores_the_clock_key_at_noon`: `left: 7000.0 right: 12000.0`. The first horizon-ramp version also failed continuity: `key illuminance jumps at hour 6.0699997: 795.18256 to 1061.1969`; the 10° ramp passes the 0.01 h sweep.
- AC2 after Task 2: rebuilt `gui build dcdbd27` without `-dirty`; fresh daemons on ports 7533 and 7534 paused at tick 120. The no-`--clock` and `--clock 22` captures both printed the creation range figures, with `clock=22.00 (capture default)` and `clock=22.00 (--clock)` respectively. `cmp` of EACH against `control-fc3dd08-a.png` exited 0 (byte identical).
- Task 3 RED: `hourly_light_table_keeps_night_exact_and_reaches_the_provisional_day`: `left: 0.0 right: 1.0` (day weight at noon). `sky_and_ambient_change_smoothly_over_each_hundredth_hour`: `the dawn sweep must contain a real sky change` (both sides were the night sky). `noon_sky_and_distance_fog_share_the_day_colour`: night `Srgba(0.019607844, 0.047058824, 0.10980392)` vs day `Srgba(0.43137255, 0.60784316, 0.8039216)`. `noon_stars_and_aurora_fade_from_the_live_shared_materials`: `star and aurora handles must reach the live app`. `noon_haze_ambient_survives_f4_off_and_on`: night ambient/intensity `(108,128,170), 1.875` vs day `(190,210,235), 5.0`. `noon_rim_materials_dissolve_toward_the_live_sky`: night rim target vs day sky. All subsequently green. Mutation results are pending the committed table run.
- AC2 after Task 3: rebuilt `gui build 947e056` without `-dirty`; fresh daemons on ports 7535 and 7536 paused at tick 120. The no-`--clock` and `--clock 22` captures each printed `warm-lit pixels=23433 ground-median-luminance=69 near-white-area=0.3906% blown-pool=0.3637% p99-luminance=153.5 resolution=1280x720`, followed by their correct `clock=22.00` source. `cmp` of EACH against `control-fc3dd08-a.png` exited 0.

### Completion Notes List

- Task 0: Controls filed as `control-fc3dd08-a.png` and `control-fc3dd08-b.png`; same-build floor is zero.
- Task 1: `ClockPin` is initialized in `projection_systems` and configured from `--clock` or the capture default. The range line uses `current_hour()` and identifies an explicit pin or capture default. Repointed affected existing mutation rows in 9.1 and 10.7. The new rows are pending the post-commit mutation run.
- Task 2: One installed directional light follows the sun from 06:00–18:00 and moon otherwise, with zero illuminance at both horizons. At 22:00 it calls the original `sun_direction()` and night table exactly. F8 restores the current key budget. AC2 capture comparison and mutation rows are pending the post-commit checks.
- Task 3: The hourly table drives ClearColor, fog, shared star and aurora materials, rim materials when sky colour changes, camera ambient, and haze ambient. F4-off/on restores the current haze after deferred commands apply. Exact #113 comment for the orchestrator to post: "11.3 settled this as: the sky is authored per hour, not exposed. ClearColor, DistanceFog.color and the rim dissolve target follow the same clock sky colour; Exposure remains 10.5." The comment was not posted here.

### File List

- `_bmad-output/implementation-artifacts/11-3-night-falls-day-breaks.md`
- `_bmad-output/implementation-artifacts/11-3-signoff/control-fc3dd08-a.png`
- `_bmad-output/implementation-artifacts/11-3-signoff/control-fc3dd08-b.png`
- `crates/gui/src/clock.rs`
- `crates/gui/src/appearance.rs`
- `crates/gui/src/atmosphere.rs`
- `crates/gui/src/project.rs`
- `_bmad-output/implementation-artifacts/11-3-signoff/boot-947e056-default.png`
- `_bmad-output/implementation-artifacts/11-3-signoff/boot-947e056-explicit.png`
- `_bmad-output/implementation-artifacts/mutations/11-3-night-falls-day-breaks.sh`
- `_bmad-output/implementation-artifacts/11-3-signoff/boot-dcdbd27-default.png`
- `_bmad-output/implementation-artifacts/11-3-signoff/boot-dcdbd27-explicit.png`
- `crates/gui/src/lib.rs`
- `crates/gui/src/ingest.rs`
- `crates/gui/src/capture.rs`
- `crates/gui/tests/capture.rs`
- `_bmad-output/implementation-artifacts/mutations/9-1-the-frame-stops-blowing-out.sh`
- `_bmad-output/implementation-artifacts/mutations/10-7-the-sun-lights-the-valley.sh`

## Change Log

| Date | Change |
| --- | --- |
| 2026-09-24 | Story created. Rulings taken (day length 24,000; moving moon, captures pin `--clock`; day artifact gated in Task 4; flat table-driven sky, `Atmosphere` probed and not adopted). Control measured on `53ba44e` (bit-identical pair, RED seen); creation probe in an isolated worktree (8 frames, filed). Adversarial validation pass: 1 critical, 4 high, 7 medium and 4 low findings applied (provisional day table plus two sittings; AC4 swap exemption; dusk at 17.5; AC8 keeps the band; AC9 bars from measurement; `ClockPin` in `projection_systems`; float hour; F4 haze re-writer). |
| 2026-09-24 | Task 0 controls on `fc3dd08` matched the creation range and each other byte for byte. |
| 2026-09-24 | Task 1 added the tick-derived hour, explicit and capture-default pinning, and capture clock reporting. |
| 2026-09-24 | Task 2 moved the single key light through sun and moon arcs and made F8 restore the current hour's key. |
| 2026-09-24 | Task 3 made sky, stars, aurora, fog, rim, and haze ambient follow the hourly light table. |
