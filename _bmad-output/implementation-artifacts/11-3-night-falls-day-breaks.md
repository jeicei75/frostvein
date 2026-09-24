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
- [x] **Task 4 — the opening artifact, and the STOP** (AC12)
  - [x] **4a.** Render ≥ 3 candidate day tables at `--clock 12` and one dusk at `--clock 17.5`
        (the sun still ~5° up — at 18.25 the key has ramped dark and the frame is ambient-only;
        tell Wolf that the key goes dark AT the horizon, so the warm light is the last hour before
        it). Each candidate is a temporary edit of the PROVISIONAL `day_lighting()` literals in an
        isolated `git worktree` — never in the shared tree ([[probe-sabotage-leaks-into-wolfs-build]])
        — boot framing, all effects on, fresh `simd --pause-at 120` each. File each frame with
        its table and range-check line under `11-3-signoff/`. Start from p3 (above).
  - [x] **STOP. Hand the frames to Wolf.** Record his words verbatim in the Dev Agent Record.
        The session resumes at 4b.
  - [x] **4b.** Write the picked table into `day_lighting()`, drop the PROVISIONAL marker, and pin
        its literals in `appearance_tables_pin_the_cold_boot_palette` (`appearance.rs:330`) beside
        the night literals. `night_lighting()`'s literals do not change.
  - [x] **4c (Wolf's sitting-1 speed ruling).** Add permanent gui `+`/`-` and numpad twins to step the reported daemon speed through Paused, Normal and Fast; refuse `--static-world`, reconcile Space, and cover the live command path and keymap.
- [x] **Task 5 — the instrument, and its test** (AC8, AC9)
  - [x] The instrument is the capture at a pinned hour:
        `gui <port> --headless --static-world --lights-steady --subdiv 4 --frames 160 --clock <h> --capture <png>`,
        read by its own range-check line (now carrying `clock=`) and by `11-2-signoff/sharpness.py`.
  - [x] Its test is AC9's guard. Run the RED in Verification **before** accepting its green.
  - [x] Run every existing rendered guard UNEDITED (`cargo test -p gui --test pixel_guard -- --ignored`)
        — the capture default pin is what keeps them valid (AC2).
- [x] **Task 6 — docs** (AC5)
  - [x] `docs/tech-art-guidelines.md`: a day row beside the night rows in the Lights table
        (`:48-78`), the clock constants, and the sky-follows-the-clock rule; strike
        "The boot frame is a night scene" (`:160`) into "boots at 22:00"; mark #113's line (`:215`)
        with how 11.3 settled it.
  - [x] `README.md`: `--clock` in the flag table (`:238-252`) with the capture default pin, and
        the seat's Fast route (`tui` + `+`) under "At the vehicle".
- [x] **Task 7 — the sitting** (AC13)
  - [x] Write `11-3-signoff/vehicle-card.md` in the seat's form (below). Questions for Wolf: the
        night vs the approved moonlit camp; the day vs AC12's frame; the dusk; AO at noon (#106);
        NFR6 at noon fullscreen.
- [x] **Task 8 — mutations and the gate** (AC14)
  - [x] `mutations/11-3-night-falls-day-breaks.sh`, at minimum: the hour ignores the tick; `--clock`
        parsed but not applied; the key's direction not driven; a below-horizon key left lit (drop
        the horizon ramp); `apply_lighting_toggles` restores `night_lighting()` at noon; stars not
        faded; the rim target left on the night sky; `DistanceFog.color` left on the night sky;
        `BOOT_HOUR` moved; a capture without `--clock` follows the tick; the hour steps in whole
        hours (integer division); F4-on restores the night haze ambient at noon.
  - [x] Every row `assert s.count(old) == 1`. Run `audit-mutations.py` **after** `cargo fmt`.

### Review Findings

Code review 2026-09-24, run 1 on `d917314` (Claude Opus 5.5 orchestrator). Four layers ran, and each
had its own warm `CARGO_TARGET_DIR`. **Every layer ran cargo and the binaries, so there is no
coverage hole.** R1's territories do not name `crates/gui`, so they were reassigned:
- Blind Hunter (Sonnet) took the pure-logic half: `clock`, `appearance`, `atmosphere`, `project`.
- Edge Case Hunter (Sonnet) took the wiring half: `ingest`, `capture`, `command`, and the tests.
- The Acceptance and Feature Auditors (Opus) reviewed the whole diff.
- The orchestrator reviewed the mutation scripts, docs, README and records inline.

Each finding is labelled with its source layer, then this workflow's HIGH/MED/LOW tier.
Totals: 2 decision-needed, 3 patch, 13 defer, 3 dismissed. There are no HIGH findings. After Wolf's rulings: 4 patch (left as action items) and 14 defer.
Review cost: $28.97 over 457 turns, of which subagents were 70.2% of tokens. The reaper then
reclaimed 146.4 GB under `/tmp`. No stray daemons or pollers were left behind.

**What ran, and what did not.** The Feature Auditor ran a probe build with the capture pin
removed against a free-running daemon. This is the first time anyone observed the UNPINNED
seat path live.
- Hours 18.20→19.25: the sky ramped `[42,63,91]→[5,12,28]` and the stars and aurora came back.
- Hours 8.67→9.76: a day frame rendered.

Both auditors confirmed that the boot capture, and the `--clock 22` capture, are
`cmp`-identical to the control, and that `--clock 12` is `cmp`-identical to
`approved-day-bb893b2.png`. **Not proven:** the `+`/`-` keys in a real window, since no
headless instrument can press keys. The full cycle at Fast on Wolf's seat is not proven either.
**AC13 stays OPEN.**

- [x] [Review][Defer] **Predawn frames trip the AC8 value floor, and no one recorded or ruled on it** (feature, MED; issue **#125**) — deferred by Wolf, 2026-09-24: settle it in #125 with the frames in hand; the seat never runs the capture check
  - Pinned captures at `--clock` 4.0, 4.5, 5.0 and 5.2 exit 101: the ground median reads 54/52/51/52, below the 55 floor. The orchestrator re-ran 4.5 and got 52, a panic at `capture.rs:1563`.
  - 3.5 reads 55 and 5.4 reads 57. The evening is close as well: 18.7, 18.8 and 18.9 read 57, 56 and 56.
  - AC8 says the band "applies at every hour", and any trip gets Wolf's ruling. The dev checked only 22, 12 and 17.5.
  - Probable cause: the moving moon sits low at about 137–148° azimuth, and day ambient starts only at 05:00. The frame reads as a dim night, not a black field.
  - Options: (a) a per-hour skip of the value floor in the `band_applies` shape; (b) retune the moon's arc or ramp, or start ambient earlier; (c) defer to a follow-up story.
- [ ] [Review][Patch] **Dusk changed after Wolf's live pick, and the record understates it** (accept, MED; Wolf ruled 2026-09-24: KEEP the 25° ramp, and disclose it in `candidates.md` and vehicle-card Q3, with the numbers, so he judges dawn and dusk live at AC13) [`_bmad-output/implementation-artifacts/11-3-signoff/vehicle-card.md:55-56`, `11-3-signoff/candidates.md`]
  - Wolf picked A live on `probe-11-3-day-toggle`, which had a 10° sun ramp (`198776e`). `7ad8b32` then widened the sun ramp to 25° to meet AC4's literal 100-lux bar.
  - At 17:30 the key falls from about 6,400 to about 1,350 lux (−79%). The sun is below full strength before about 08:35 and after about 15:25.
  - The record says "the wider ramp barely moves it", but that is measured on ground median alone (109→108). The frame is not byte-identical to the filed candidate-A dusk, and its p99 moves 180.2→169.0.
  - `candidates.md` still says "horizon ramp over the last 10°", and vehicle-card Q3 does not mention the change.
  - Options: (a) keep 25°, and disclose it on the card and in `candidates.md` so Wolf judges dawn and dusk at AC13; (b) reopen AC4's bar, for example allowing key illuminance a bar of its own.
- [ ] [Review][Patch] **Mutation row "a default capture follows the wire tick" no longer sabotages the capture default** (accept + orchestrator, MED) [`_bmad-output/implementation-artifacts/mutations/11-3-night-falls-day-breaks.sh:82-88`]
  - Its payload is byte-identical to the last row's, "rendered noon ignores the explicit clock pin" (`current_hour` ignores every pin).
  - The Task 8 defect lives at `ingest.rs` `.or(args.capture.as_ref().map(|_| BOOT_HOUR))`. `9bfd18d` had the right payload; `758a561` and `8ba95ce` drifted away from it.
  - The Dev Agent Record's kill line describes a payload that no longer exists.
  - Fix: restore the capture-default payload, then re-mutate that row alone.
- [ ] [Review][Patch] **The vehicle card's pinned noon capture may exit 101 on the seat GPU, with no warning** (feature, MED) [`_bmad-output/implementation-artifacts/11-3-signoff/vehicle-card.md:40-41`]
  - Devpod noon near-white is 0.8519% against a 0.9461% ceiling, only 0.094 pp of headroom.
  - The delivery GPU reads bright-tail near-white 0.4–0.6 pp higher (see memory "capture ceiling is venue-sited").
  - The card says "the capture band applies to both". It should predict a possible exit 101 at noon and say that a 101 there is a venue offset, not a regression.
- [ ] [Review][Patch] **The card's live night comparison is against a moved moon** (feature, LOW; patched because the card is being edited anyway) [`_bmad-output/implementation-artifacts/11-3-signoff/vehicle-card.md:21-24`]
  - "The unpinned client begins at 22:00" is true only at daemon tick 0. Every minute before the first frame moves the clock 0.6 h and the moon 9° of azimuth.
  - Judge the night against the approved moonlit camp from the pinned 22:00 capture. Use the live run for the cycle only.
- [x] [Review][Defer] `lighting_at()` blends `directional` and `directional_illuminance` that nothing reads (`appearance.rs:94-96`). The key takes those only from `key_at`. So there are two sources for the key, and they disagree between 05:00 and 07:00 (accept LOW) [crates/gui/src/appearance.rs:94] — deferred, YAGNI trap for a future reader
- [x] [Review][Defer] The new `CaptureClock` struct sits between `capture_after_frames`' doc and lint comments and the function, so the comment now documents the struct. A meaningless `#[allow(clippy::too_many_arguments)]` also sits on the struct (accept LOW) [crates/gui/src/capture.rs:934-943] — deferred
- [x] [Review][Defer] In the tech-art Lights table, the day rows and the D row were inserted between `night_lighting().directional` and its "↳ before 10.8 … (superseded)" row, which now reads as belonging to D (accept + orchestrator LOW) [docs/tech-art-guidelines.md:61-65] — deferred
- [x] [Review][Defer] AC5 tests compare ClearColor, fog and rim only at 12:00 and 22:00. No live-app sweep across hours checks that the three stay equal (accept LOW) [crates/gui/src/ingest.rs:4020] — deferred; the code is structurally one value per frame
- [x] [Review][Defer] The AC6/AC7 tests call `current_hour()` on swapped resources instead of updating the live app, and the unpinned test builds two apps, not one app fed two snapshots. AC7's RED line `clock=22.1x` was never recorded (accept LOW) [crates/gui/src/ingest.rs:2698] — deferred
- [x] [Review][Defer] AC10's `--lights-off sun`/`ambient` "at every hour" is untested away from the boot hour. It is correct by construction, since `apply_lighting_toggles` is the single writer (accept LOW) [crates/gui/src/ingest.rs:1811] — deferred
- [x] [Review][Defer] The AC4 sweep does not check key colour or direction continuity while the key is lit. Both are constant or continuous by construction (accept LOW) [crates/gui/src/atmosphere.rs:352] — deferred
- [x] [Review][Defer] At the 06:00 and 18:00 swaps, key colour and direction change while the outgoing body still carries about 0.006 lux. That is a literal breach of AC4's "exempt only while illuminance is 0", but invisible (blind LOW) [crates/gui/src/atmosphere.rs:262] — deferred
- [x] [Review][Defer] Pressing `+` twice before the daemon echoes the speed re-sends Normal instead of reaching Fast. `step_speed` reads the echoed `Mirror::speed()`. The TUI does the same (`tui/src/view.rs:484-491`), which matches the ruled "mirror the TUI" (edge LOW) [crates/gui/src/command.rs:225] — deferred
- [x] [Review][Defer] `--clock 23.9999999` rounds to f32 24.0 and is rejected as "0 <= hour < 24" (edge LOW) [crates/gui/src/ingest.rs:1096] — deferred
- [x] [Review][Defer] The README says "`+` steps Normal to Fast" and omits Paused→Normal. The controls table is correct (orchestrator LOW) [README.md:182] — deferred
- [x] [Review][Defer] The `sprint-status.yaml` 11-3 comment block still reads "DEV STARTED -> in-progress … sitting 1" while the status is `review` (orchestrator LOW) [_bmad-output/implementation-artifacts/sprint-status.yaml:2460] — deferred
- [x] [Review][Defer] The probe binary built from a `git archive` copy stamped itself `d917314` with no `-dirty`, so an archive build's stamp cannot vouch for its content (feature LOW, not this story's code) [crates/gui build stamp] — deferred, pre-existing

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

- Sitting 2, AC4 RED: changing the key sweep bound from 240 lux (2% of the day value) to 100 lux (2% of the 7,000→12,000 range) failed `lit_key_never_points_up_or_jumps_in_illuminance`: `key illuminance jumps at hour 6.16: 794.86163 to 897.2637`. Widening the SUN horizon ramp to 25° made the 0.01 h sweep green; the moon keeps its 10° ramp and arc. The same sweep already checks sky R/G/B against 2% of each channel's night-to-day range, ambient R/G/B likewise, ambient brightness against 2% of 2,500 = 50, and star/aurora fade against 2% of 1 = 0.02. Equal night/day star and aurora table colours are invariant; their fade factors are the driven values.
- AC2 after the sun ramp: rebuilt `gui build 7ad8b32` without `-dirty`, fresh daemons on ports 7541/7542 paused at tick 120; captures `boot-7ad8b32-{default,explicit}.png` each compared byte-identical to `control-fc3dd08-a.png`.
- Short-circuit removal RED: after deleting the boot-hour branch, the existing boot equality passed through the moon arc. A temporary +1° moon azimuth change made `key_arc_uses_the_approved_boot_direction_and_day_table` fail: `left: Vec3(0.7187084, -0.3033679, 0.6256407) right: Vec3(0.7295178, -0.3033679, 0.6130022)`. Restoring the original moon formula made it green. The fresh AC2 capture pair is still pending.
- AC2 after short-circuit removal: rebuilt `gui build 0e6a695` without `-dirty`, fresh daemons on ports 7543/7544 paused at tick 120; `boot-0e6a695-{default,explicit}.png` each compared byte-identical to `control-fc3dd08-a.png`.
- Speed keys RED, before implementation: `speed_keys_step_from_the_daemon_speed_and_ignore_the_ends` failed `Equal from Paused left: "" right: "{\"type\":\"set_speed\",\"speed\":\"normal\"}"`; `speed_keys_refuse_static_world_and_space_uses_the_new_pause_state` failed its seat case with the same empty command versus Normal. Both passed after registering `step_speed` before `send_commands`. The tests read the actual socket command for all four steps, both ignored ends, the static-world refusal, and Space immediately after `+` from Paused. Mutation rows remain pending.
- AC2 after speed-key wiring: rebuilt `gui build 8d03dca` without `-dirty`, fresh daemons on ports 7545/7546 paused at tick 120; `boot-8d03dca-{default,explicit}.png` each compared byte-identical to `control-fc3dd08-a.png`.
- Task 5 AC9 RED in Verification: temporarily made `current_hour` ignore `ClockPin`; `night_turns_into_day_on_the_rendered_frame` failed `noon ground must exceed night by more than 10: 69 -> 69` (sky-stars 1.8736 → 1.8736). Restored the source, rebuilt through `cargo test`, and the guard passed on real frames: ground 69 → 161; sky-stars 1.8823 → 0.5127. Further capture repeats and the full ignored suite are pending.
- Task 5 instrument, four fresh `simd --pause-at 120` captures PER hour on `gui build e237f62` (no `-dirty`), all `--headless --static-world --lights-steady --subdiv 4 --frames 160`: `clock-e237f62-h22-{1,2,3,4}.png` on ports 7552/7554/7556/7558, and `clock-e237f62-h12-{1,2,3,4}.png` on ports 7553/7555/7557/7559. Every 22:00 line was identical: warm-lit 23,433; ground 69; near-white 0.3906%; blown-pool 0.3637%; p99 153.5; `clock=22.00 (--clock)`. Every noon line was identical: warm-lit 10,218; ground 161; near-white 0.8519%; blown-pool 0.5068%; p99 197.4; `clock=12.00 (--clock)`. All exited 0 under the unchanged band. Same-hour SHA-256 hashes match across all four files (night `7221b15c85fe1eceac4afd8205c49adb9ffb601391b4ed8b35e3159653f185b3`; noon `cfdf5e0ed6838488ebca93f5e8df4fb56ab2b8735eafdaf2c4554b2731686103`), so each hour's measured same-build floor is 0. `sharpness.py` reads sky-stars `lap_mean` night 1.8812 → noon 0.5228, a 72% fall; the AC9 guard's pixel window computes 1.8823 → 0.5127 using only the interior pixels.
- Task 5 full rendered suite: `RUST_TEST_THREADS=2 cargo test --offline -p gui --test pixel_guard -- --ignored` passed **12 passed, 0 failed** in 1679.00 s. The existing guards were unedited; `pixel_guard.rs` adds only AC9's guard.
- Speed-state test strengthening RED: starting the local `SimPaused` flag in the daemon's Paused state, then removing the speed key's update made `speed_keys_refuse_static_world_and_space_uses_the_new_pause_state` fail: `Space after + must pause the newly running sim; left: {"type":"set_speed","speed":"normal"} right: {"type":"set_speed","speed":"paused"}`. Restoring the update made it green. The matching mutation row is pending Task 8.
- Sitting 2, Task 4b RED: temporarily changed only the candidate A sky literal to `(111,155,205)`; `appearance_tables_pin_the_cold_boot_palette` failed at `appearance.rs:570`: `left: [111, 155, 205] right: [110, 155, 205]`. Restored `(110,155,205)` and the focused test passed. The day literal pin's mutation row is pending Task 8.

- Task 0: `cargo build --offline -p gui -p simd` passed; `./target/debug/gui --version` printed `gui build fc3dd08` without `-dirty`.
- Fresh daemons on ports 7531 and 7532 each paused at tick 120. Both captures printed `warm-lit pixels=23433 ground-median-luminance=69 near-white-area=0.3906% blown-pool=0.3637% p99-luminance=153.5 resolution=1280x720`. `cmp` of the two PNGs exited 0.
- Task 1 RED: `tick_clock_moves_fractionally_and_wraps`: `left: 22.0 right: 22.5` (hour ignored tick). `clock_flag_accepts_hours_and_rejects_out_of_range_values`: `called Result::unwrap() on an Err value: invalid port`. `clock_pin_reaches_the_live_app_and_capture_defaults_to_boot`: `left: None right: Some(None)`. `an_unpinned_seat_follows_two_wire_snapshots_one_hour_apart`: `left: 0.0 right: 1.0`. `capture_clock_note_names_the_rendered_hour_and_source`: `left: "" right: " clock=12.00 (--clock)"`. All subsequently green. Mutation rows and results will be recorded after the committed table runs.
- Task 2 RED: `key_arc_uses_the_approved_boot_direction_and_the_provisional_noon_table`: `assertion failed: noon_direction.y < night_direction.y`. `lit_key_never_points_up_or_jumps_in_illuminance`: `both keys are dark at dawn; left: 7000.0 right: 0.0`. `clock_drives_the_installed_key_direction_color_and_illuminance`: installed `Vec3(0.7295178, -0.3033679, 0.61300206)` vs noon `Vec3(0.5864819, -0.6427876, 0.49281135)`. `f8_restores_the_clock_key_at_noon`: `left: 7000.0 right: 12000.0`. The first horizon-ramp version also failed continuity: `key illuminance jumps at hour 6.0699997: 795.18256 to 1061.1969`; the 10° ramp passes the 0.01 h sweep.
- AC2 after Task 2: rebuilt `gui build dcdbd27` without `-dirty`; fresh daemons on ports 7533 and 7534 paused at tick 120. The no-`--clock` and `--clock 22` captures both printed the creation range figures, with `clock=22.00 (capture default)` and `clock=22.00 (--clock)` respectively. `cmp` of EACH against `control-fc3dd08-a.png` exited 0 (byte identical).
- Task 3 RED: `hourly_light_table_keeps_night_exact_and_reaches_the_provisional_day`: `left: 0.0 right: 1.0` (day weight at noon). `sky_and_ambient_change_smoothly_over_each_hundredth_hour`: `the dawn sweep must contain a real sky change` (both sides were the night sky). `noon_sky_and_distance_fog_share_the_day_colour`: night `Srgba(0.019607844, 0.047058824, 0.10980392)` vs day `Srgba(0.43137255, 0.60784316, 0.8039216)`. `noon_stars_and_aurora_fade_from_the_live_shared_materials`: `star and aurora handles must reach the live app`. `noon_haze_ambient_survives_f4_off_and_on`: night ambient/intensity `(108,128,170), 1.875` vs day `(190,210,235), 5.0`. `noon_rim_materials_dissolve_toward_the_live_sky`: night rim target vs day sky. All subsequently green. Mutation results are pending the committed table run.
- AC2 after Task 3: rebuilt `gui build 947e056` without `-dirty`; fresh daemons on ports 7535 and 7536 paused at tick 120. The no-`--clock` and `--clock 22` captures each printed `warm-lit pixels=23433 ground-median-luminance=69 near-white-area=0.3906% blown-pool=0.3637% p99-luminance=153.5 resolution=1280x720`, followed by their correct `clock=22.00` source. `cmp` of EACH against `control-fc3dd08-a.png` exited 0.
- Mutation run: `scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/11-3-night-falls-day-breaks.sh` ran after commit `9bfd18d`. All 19 rows KILLED. The default-capture row first died on the earlier pin-resource assertion; after `758a561` and `8ba95ce`, its focused rerun KILLED on `a running capture must stay at 22 after the wire advances 1,000 ticks`. `scripts/audit-mutations.py` after `cargo fmt` reported `635 rows, every literal still matches its target`.
- Final rim precision RED: `noon_rim_materials_dissolve_toward_the_live_sky` at 22:00 printed `night rim target must equal the live sky exactly`; material `Srgba(0.019607842, 0.04705882, 0.109803915)` versus sky `Srgba(0.019607844, 0.047058824, 0.10980392)`. Returning the single sky value at the last rim level fixed it. The `night rim target misses the exact sky colour` row KILLED on that exact assertion. `scripts/audit-mutations.py` after `cargo fmt` reported 636 rows matching source.
- AC2 after final rim fix: rebuilt `gui build b2b8f0f` without `-dirty`; fresh daemons on ports 7537 and 7538 paused at tick 120. Both final captures printed the creation figures (`23433`, `69`, `0.3906%`, `0.3637%`, `153.5`, `1280x720`) plus `clock=22.00 (capture default)` or `clock=22.00 (--clock)`. `cmp` of EACH against `control-fc3dd08-a.png` exited 0. This is the final code's AC2 pair.
- Self-gate attempt 1: `codex review --base main` started but could not inspect any diff. Every shell invocation failed before execution with `error building bubblewrap command: app-server socket directory must be a user-owned directory with mode 0700`. It returned no code findings and cannot be counted as a successful review pass.

- Task 8 post-commit mutation run: `RUST_TEST_THREADS=2 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/11-3-night-falls-day-breaks.sh` completed with all 37 rows KILLED. The sitting-2 RED-to-row map is: `appearance_tables_pin_the_cold_boot_palette` (sky `[111,155,205]` vs `[110,155,205]`) → five `approved day ... drifts` rows; `lit_key_never_points_up_or_jumps_in_illuminance` (`6.16: 794.86163 to 897.2637`) → `sun horizon ramp violates the literal 100 lux bar`; `key_arc_uses_the_approved_boot_direction_and_day_table` (`Vec3(0.7187084,-0.3033679,0.6256407)` vs `Vec3(0.7295178,-0.3033679,0.6130022)`) → `boot direction no longer comes from the exact moon arc`; `speed_keys_step_from_the_daemon_speed_and_ignore_the_ends` (`Equal from Paused` sent empty before implementation) → four transition, end-press and two numpad rows; `speed_keys_refuse_static_world_and_space_uses_the_new_pause_state` (missing Normal before implementation, then stale Space sent Normal rather than Paused under sabotage) → static-world and stale-pause rows; `night_turns_into_day_on_the_rendered_frame` (`69 -> 69`) → rendered noon pin row. Each targeted test went red for the intended behavior and green after restoration.
- `cargo fmt --all` then `python3 scripts/audit-mutations.py`: `653 rows, every literal still matches its target (11 rows carry no count guard)`. The 17 new rows all have `assert s.count(old) == 1`; the 11 unguarded rows are existing repo backlog.
- Last explicit gate: `RUST_TEST_THREADS=2 scripts/gate.sh --fast` GREEN (72 s). This fast tier skipped `simd/tests/serve.rs` and the rendered pixel guards; the latter were run separately and passed 12/12. The orchestrator's earlier full gate was GREEN on `bb893b2`, before sitting-2 changes. A full post-sitting-2 gate was not run here; Wolf's orchestrator runs it.

```text
================ MUTATION RESULTS ================
hour ignores the wire tick                                   KILLED
clock flag parses but never reaches the pin                  KILLED
installed key direction stays at the boot aim                KILLED
the key stays lit at the horizon                             KILLED
F8 restores the night budget at noon                         KILLED
stars retain their night colour at noon                      KILLED
rim materials keep the night sky target                      KILLED
distance fog keeps the night sky                             KILLED
the boot hour moves from 22                                  KILLED
a default capture follows the wire tick                      KILLED
hour advances in whole-hour steps                            KILLED
F4-on reinserts night haze after the clock writer            KILLED
clock parser admits the excluded upper bound                 KILLED
capture range line loses its clock field                     KILLED
capture clock note reverses its pin source                   KILLED
current_hour ignores an unpinned snapshot                    KILLED
hourly table stays night at noon                             KILLED
sky and ambient jump at dawn                                 KILLED
aurora remains opaque at noon                                KILLED
night rim target misses the exact sky colour                 KILLED
approved day sky drifts from candidate A                     KILLED
approved day ambient tint drifts                             KILLED
approved day ambient budget drifts                           KILLED
approved day key tint drifts                                 KILLED
approved day key budget drifts                               KILLED
sun horizon ramp violates the literal 100 lux bar            KILLED
boot direction no longer comes from the exact moon arc       KILLED
plus from Paused fails to request Normal                     KILLED
plus from Normal fails to request Fast                       KILLED
minus from Fast fails to request Normal                      KILLED
minus from Normal fails to request Paused                    KILLED
speed end presses send an unwanted command                   KILLED
numpad plus fails to reach speed stepping                    KILLED
numpad minus fails to reach speed stepping                   KILLED
speed key resumes a static-world capture                     KILLED
plus leaves Space using stale paused state                   KILLED
rendered noon ignores the explicit clock pin                 KILLED

All mutations killed.
```

### Completion Notes List

- Sitting 2, Task 4b: Wolf's approved candidate A was already the provisional table; removed the provisional marker and pinned its sky, ambient, ambient brightness, directional colour and illuminance as independent literals beside the unchanged night pins.
- Wolf's speed ruling is filed as Task 4c in Tasks/Subtasks. `step_speed` reads `Mirror::speed()` for each step, updates `SimPaused` so Space after `+` pauses, and uses the existing `SetSpeed { at_tick: None }`. `restore_speed_on_exit` remains Normal for static-world sessions and its configured-app socket test remains green.
- Task 6: documented candidate A's approved day table, clock constants and shared sky colour; D “pale overcast” is a named UNBUILT future weather table only. README now names `--clock`, its capture default, and the gui's own Fast key.
- Task 7: wrote the vehicle card with the seat's launcher form, a fresh daemon before every static capture, absolute `.bin` outputs, Fast through gui `+`, and Wolf's look/AO/NFR6 questions. Wolf's AC13 sitting has not been observed in this dev run.

- Task 0: Controls filed as `control-fc3dd08-a.png` and `control-fc3dd08-b.png`; same-build floor is zero.
- Task 1: `ClockPin` is initialized in `projection_systems` and configured from `--clock` or the capture default. The range line uses `current_hour()` and identifies an explicit pin or capture default. Repointed affected existing mutation rows in 9.1 and 10.7. The new rows are pending the post-commit mutation run.
- Task 2: One installed directional light follows the sun from 06:00–18:00 and moon otherwise, with zero illuminance at both horizons. At 22:00 it calls the original `sun_direction()` and night table exactly. F8 restores the current key budget. AC2 capture comparison and mutation rows are pending the post-commit checks.
- Task 3: The hourly table drives ClearColor, fog, shared star and aurora materials, rim materials when sky colour changes, camera ambient, and haze ambient. F4-off/on restores the current haze after deferred commands apply. Exact #113 comment for the orchestrator to post: "11.3 settled this as: the sky is authored per hour, not exposed. ClearColor, DistanceFog.color and the rim dissolve target follow the same clock sky colour; Exposure remains 10.5." The comment was not posted here.
- RED-to-mutation map (each row KILLED): `tick_clock_moves_fractionally_and_wraps` → `hour ignores the wire tick`, `the boot hour moves from 22`, `hour advances in whole-hour steps`; `clock_flag_accepts_hours_and_rejects_out_of_range_values` → `clock parser admits the excluded upper bound`; `clock_pin_reaches_the_live_app_and_capture_defaults_to_boot` → `clock flag parses but never reaches the pin`, `a default capture follows the wire tick`; `an_unpinned_seat_follows_two_wire_snapshots_one_hour_apart` → `current_hour ignores an unpinned snapshot`; `capture_clock_note_names_the_rendered_hour_and_source` → `capture clock note reverses its pin source`; the strengthened `the_range_check_line_reports_the_frame_shape` → `capture range line loses its clock field` (mutation RED at `capture.rs:1952`, clock assertion); `key_arc_uses_the_approved_boot_direction_and_the_provisional_noon_table` → noon-table and direction RED recorded above, while the installed path is killed by `installed key direction stays at the boot aim`; `lit_key_never_points_up_or_jumps_in_illuminance` → `the key stays lit at the horizon`; `clock_drives_the_installed_key_direction_color_and_illuminance` → `installed key direction stays at the boot aim`; `f8_restores_the_clock_key_at_noon` → `F8 restores the night budget at noon`; `hourly_light_table_keeps_night_exact_and_reaches_the_provisional_day` → `hourly table stays night at noon`; `sky_and_ambient_change_smoothly_over_each_hundredth_hour` → `sky and ambient jump at dawn`; `noon_sky_and_distance_fog_share_the_day_colour` → `distance fog keeps the night sky`; `noon_stars_and_aurora_fade_from_the_live_shared_materials` → `stars retain their night colour at noon`, `aurora remains opaque at noon`; `noon_rim_materials_dissolve_toward_the_live_sky` → `rim materials keep the night sky target`; `noon_haze_ambient_survives_f4_off_and_on` → `F4-on reinserts night haze after the clock writer` (mutation RED at the final F4-on assertion). No verification hook was bypassed.
- The strengthened night-rim assertion in `noon_rim_materials_dissolve_toward_the_live_sky` → `night rim target misses the exact sky colour` KILLED. The row was run after commit `b2b8f0f`; its failure line was `night rim target must equal the live sky exactly`.

```text
================ MUTATION RESULTS ================
hour ignores the wire tick                                   KILLED
clock flag parses but never reaches the pin                  KILLED
installed key direction stays at the boot aim                KILLED
the key stays lit at the horizon                             KILLED
F8 restores the night budget at noon                         KILLED
stars retain their night colour at noon                      KILLED
rim materials keep the night sky target                      KILLED
distance fog keeps the night sky                             KILLED
the boot hour moves from 22                                  KILLED
a default capture follows the wire tick                      KILLED
hour advances in whole-hour steps                            KILLED
F4-on reinserts night haze after the clock writer            KILLED
clock parser admits the excluded upper bound                 KILLED
capture range line loses its clock field                     KILLED
capture clock note reverses its pin source                   KILLED
current_hour ignores an unpinned snapshot                    KILLED
hourly table stays night at noon                             KILLED
sky and ambient jump at dawn                                 KILLED
aurora remains opaque at noon                                KILLED
night rim target misses the exact sky colour                 KILLED

All mutations killed.
```


### Orchestrator verification (Claude Opus 5.5, 2026-09-24)

Codex exited 1 on its **usage limit** mid-final-status-check; no hand-back message was written, the
tree was clean and every Task 0-3 commit was already down (11 commits, all `Völundr`, committer
`jeicei75`; no `--no-verify` in any commit invocation; every pre-commit fast gate GREEN). Wolf reset
the quota; a probe confirmed Codex is back. Verified rather than trusted:

- **AC2, re-captured independently** on `gui build bb893b2`, fresh `simd --pause-at 120` each: no
  `--clock` → `clock=22.00 (capture default)`, `--clock 22` → `clock=22.00 (--clock)`, both with the
  creation line verbatim, and **both `cmp`-identical to `creation-control-53ba44e-a.png`**.
- **Noon is the probe's day, byte for byte:** `--clock 12` on the real code is `cmp`-identical to
  `probe-p3-day-flat.png` (ground median 161; `sky-stars` lap_mean 1.8812 → 0.5228).
- **Scope:** `git diff 0fbb6ce..HEAD --stat` on `crates/{protocol,sim-core,client-core,simd}` and
  `crates/gui/tests/pixel_guard.rs` is empty.
- **FULL GATE GREEN on `bb893b2`, 1680 s, `RUST_TEST_THREADS=2`.**
- **Self-gate did NOT run:** `codex review --base main` died on
  `app-server socket directory must be a user-owned directory with mode 0700` (a new bubblewrap
  cause; 11.2's was the `/tmp` lock). The code review carries the full weight.
- **For the code review, not fixed here:** (1) `key_at` short-circuits `hour == BOOT_HOUR` to
  `sun_direction()`/night table, so the boot-hour tests never exercise the moon arc (the arc is exact
  there arithmetically, so the branch is redundant but hides arc mutations at 22.0);
  (2) `lit_key_never_points_up_or_jumps_in_illuminance` bounds the key step at 2 % of the DAY
  illuminance (240 lux); AC4 read literally is 2 % of |day − night| = 100 lux, and the sun's 10°
  horizon ramp steps ~190 lux per 0.01 h.
- **Task 4a** rendered by the orchestrator in an isolated worktree of `bb893b2` (4 candidates × noon
  + dusk 17.5): `11-3-signoff/candidates.md`, `candidates-contact-sheet-bb893b2.png`,
  `candidate-*-bb893b2-h*.png`. Every frame passes the existing band. Candidate D would fail AC9's
  ≥ 50 % sky bar (−7 %). **STOPPED for Wolf's pick.**

### Wolf's rulings during sitting 1 (2026-09-24, verbatim where quoted)

- Undecided between A-D from stills: *"need to see live but that requires fast mode and ABCD toggles"*.
- **Speed keys are PERMANENT, the A-D toggle is temporary:** *"ABCD is temporary .. fast-clock is
  permanent"*; *"we should have at least pause, normal, fast modes for playing and testing purposes..
  later on maybe HUD UI under minimap"*. He is NOT asking for a faster Normal. **This overrides the
  guardrail "Do NOT add a key" for speed only:** the gui gains `+` / `-` (Equal/NumpadAdd,
  Minus/NumpadSubtract) stepping Paused -> Normal -> Fast and back through the existing
  `SetSpeed`, mirroring `tui` (`view.rs:483-492`); `Space` keeps pause. No protocol/sim change.
  To be built in sitting 2 with a test and a mutation row, and reconciled with `SimPaused`
  (`command.rs:285`) so Space after `+` does not act on a stale local flag.
- The live comparison runs on a THROWAWAY branch `probe-11-3-day-toggle` @ `24693c1` (pushed, never
  merged): `T` cycles A->B->C->D, `+`/`-` step the speed; state prints to stderr only.
- **THE PICK (Task 4, AC12), judged LIVE on the probe at Fast with `T`:** *"I like them all :) well A
  or D ..hmmhm"*; on the recommendation "ship A as the day, record D as the overcast table for a future
  weather story": *"yes.. let's do that to get progress..."*. **A ships** — its table is the PROVISIONAL
  one already in `day_lighting()` (creation probe p3), so 4b drops the marker and pins those literals.
  Approved frame: `11-3-signoff/approved-day-bb893b2.png` (= `candidate-A-provisional-bb893b2-h12.png`,
  `cmp`-identical to `probe-p3-day-flat.png`). **D is recorded, not built** (a named overcast table for
  a later weather story; no code). AC9's bars stand as written against A (ground 69 -> 161, sky-stars
  -72 %).
- Wolf, same message: *"I think we need to have swappable configurations anyway to tweak without
  compiling"* — NOT in 11.3's scope (technical-preferences: no config files before a third concrete
  use; a story must require runtime change). Routed to its own issue/story.

### Orchestrator verification, sitting 2 (Claude Opus 5.5, 2026-09-24)

Codex exited 0 with a full hand-back. Verified rather than trusted:

- 10 commits `198776e..6cc4981`, all authored `Völundr`; no `--no-verify` in any commit invocation;
  no attribution trailers anywhere in `0fbb6ce..HEAD`; tree clean; Status `review` in the story and
  sprint-status.
- **Record deviation (not rewritten):** sitting 2 also set the COMMITTER to Völundr
  (`git -c user.name='Völundr' ... commit`), where the rule keeps the committer `jeicei75`. The
  commits are unpushed, but rewriting would change the SHAs that the `boot-<sha>-*.png` filenames and
  this record cite, so it is left and the next handoff prompt names the committer rule.
- **Independent captures on `gui build 6cc4981`**, fresh `simd --pause-at 120` each: no `--clock` and
  `--clock 22` both `cmp`-identical to `creation-control-53ba44e-a.png`; `--clock 12` `cmp`-identical
  to `approved-day-bb893b2.png`; `--clock 17.5` ground median 108 (109 before the 25° sun ramp — the
  dusk frame is ambient-dominated, so the wider ramp barely moves it).
- Scope: `crates/{protocol,sim-core,client-core,simd}` unchanged since `0fbb6ce`; `pixel_guard.rs` gained
  only the AC9 guard (+48 lines).
- **FULL GATE GREEN on `6cc4981`, 1822 s, `RUST_TEST_THREADS=2`.**
- Self-gate did not run (same bubblewrap socket error); the code review carries the full weight.
- Dev cost recorded: Codex $11.10 (16pp) + $8.19 (12pp), orchestrator $19.03.

### File List

- `_bmad-output/implementation-artifacts/11-3-night-falls-day-breaks.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/11-3-signoff/control-fc3dd08-a.png`
- `_bmad-output/implementation-artifacts/11-3-signoff/control-fc3dd08-b.png`
- `crates/gui/src/clock.rs`
- `crates/gui/src/appearance.rs`
- `crates/gui/src/atmosphere.rs`
- `crates/gui/src/project.rs`
- `_bmad-output/implementation-artifacts/11-3-signoff/boot-947e056-default.png`
- `_bmad-output/implementation-artifacts/11-3-signoff/boot-947e056-explicit.png`
- `_bmad-output/implementation-artifacts/11-3-signoff/boot-b2b8f0f-default.png`
- `_bmad-output/implementation-artifacts/11-3-signoff/boot-b2b8f0f-explicit.png`
- `_bmad-output/implementation-artifacts/11-3-signoff/boot-7ad8b32-default.png`
- `_bmad-output/implementation-artifacts/11-3-signoff/boot-7ad8b32-explicit.png`
- `_bmad-output/implementation-artifacts/11-3-signoff/boot-0e6a695-default.png`
- `_bmad-output/implementation-artifacts/11-3-signoff/boot-0e6a695-explicit.png`
- `_bmad-output/implementation-artifacts/11-3-signoff/boot-8d03dca-default.png`
- `_bmad-output/implementation-artifacts/11-3-signoff/boot-8d03dca-explicit.png`
- `_bmad-output/implementation-artifacts/11-3-signoff/clock-e237f62-h12-{1,2,3,4}.png`
- `_bmad-output/implementation-artifacts/11-3-signoff/clock-e237f62-h22-{1,2,3,4}.png`
- `_bmad-output/implementation-artifacts/11-3-signoff/vehicle-card.md`
- `crates/gui/src/command.rs`
- `crates/gui/tests/pixel_guard.rs`
- `_bmad-output/implementation-artifacts/mutations/11-1b-the-air-has-depth.sh`
- `docs/tech-art-guidelines.md`
- `README.md`
- `_bmad-output/implementation-artifacts/mutations/11-3-night-falls-day-breaks.sh`
- `_bmad-output/implementation-artifacts/11-3-signoff/boot-dcdbd27-default.png`
- `_bmad-output/implementation-artifacts/11-3-signoff/boot-dcdbd27-explicit.png`
- `crates/gui/src/lib.rs`
- `crates/gui/src/ingest.rs`
- `crates/gui/src/capture.rs`
- `crates/gui/tests/capture.rs`
- `_bmad-output/implementation-artifacts/mutations/9-1-the-frame-stops-blowing-out.sh`
- `_bmad-output/implementation-artifacts/mutations/10-7-the-sun-lights-the-valley.sh`
- `_bmad-output/implementation-artifacts/11-3-signoff/candidates.md`
- `_bmad-output/implementation-artifacts/11-3-signoff/approved-day-bb893b2.png`
- `_bmad-output/implementation-artifacts/11-3-signoff/candidates-contact-sheet-bb893b2.png`
- `_bmad-output/implementation-artifacts/11-3-signoff/candidate-{A-provisional,B-soft-warm,C-crisp-blue,D-pale-overcast}-bb893b2-h{12,17.5}.png`

## Change Log

| Date | Change |
| --- | --- |
| 2026-09-24 | Story created. Rulings taken (day length 24,000; moving moon, captures pin `--clock`; day artifact gated in Task 4; flat table-driven sky, `Atmosphere` probed and not adopted). Control measured on `53ba44e` (bit-identical pair, RED seen); creation probe in an isolated worktree (8 frames, filed). Adversarial validation pass: 1 critical, 4 high, 7 medium and 4 low findings applied (provisional day table plus two sittings; AC4 swap exemption; dusk at 17.5; AC8 keeps the band; AC9 bars from measurement; `ClockPin` in `projection_systems`; float hour; F4 haze re-writer). |
| 2026-09-24 | Task 0 controls on `fc3dd08` matched the creation range and each other byte for byte. |
| 2026-09-24 | Task 1 added the tick-derived hour, explicit and capture-default pinning, and capture clock reporting. |
| 2026-09-24 | Task 2 moved the single key light through sun and moon arcs and made F8 restore the current hour's key. |
| 2026-09-24 | Task 3 made sky, stars, aurora, fog, rim, and haze ambient follow the hourly light table. |
| 2026-09-24 | Verified both post-sky boot captures byte-identical to control; killed 19 sitting-1 mutation rows and tightened the running-capture tick guard. |
| 2026-09-24 | Made the last rim level equal the shared sky value at night; final boot captures stayed byte identical, and the 20th mutation row KILLED. Self-review was blocked by bubblewrap socket permissions. |
| 2026-09-24 | Orchestrator verification of Tasks 0-3 (AC2 re-captured cmp-identical, noon == probe p3, full gate GREEN 1680 s on `bb893b2`); Task 4a: four candidate day tables rendered in an isolated worktree and filed. STOPPED for Wolf's pick. |
| 2026-09-24 | Wolf ruled permanent `+`/`-` sim-speed keys in the gui (Paused/Normal/Fast via existing `SetSpeed`); throwaway probe branch `probe-11-3-day-toggle` pushed for his live A-D pick. |
| 2026-09-24 | Wolf picked candidate A live (D recorded as a future overcast table); approved frame filed; STOP released — sitting 2 (4b onward) next. |
| 2026-09-24 | Task 4b pinned Wolf's approved candidate A as hand-written day literals and removed the provisional marker. |
| 2026-09-24 | Tightened AC4's key continuity bar to 100 lux per 0.01 h and widened only the sun's horizon ramp to pass it. |
| 2026-09-24 | Removed the boot-hour key short-circuit so the approved direction is produced by the moon arc itself. |
| 2026-09-24 | Added permanent gui speed keys from Wolf's ruling using the daemon's reported speed and reconciled Space's pause state. |
| 2026-09-24 | Added AC9's ignored rendered night-to-noon guard, observed its clock-pin RED, then its green ground and sky changes. |
| 2026-09-24 | Captured four reproducible night and four reproducible noon frames; documented approved A and future overcast D, clock controls, and Wolf's vehicle sitting. |
| 2026-09-24 | All 37 story mutations KILLED; audit matched 653 rows after formatting, and the explicit fast gate passed. Moved story to review for Wolf's vehicle sitting. |
| 2026-09-24 | Orchestrator verification of sitting 2: independent AC2/noon captures cmp-identical, full gate GREEN 1822 s on `6cc4981`; committer deviation recorded; dev cost recorded. |
