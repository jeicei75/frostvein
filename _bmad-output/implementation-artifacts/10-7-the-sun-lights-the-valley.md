---
baseline_commit: e930d07
model: claude-opus-5[1m]  # Opus default; the 1M-context variant, recorded so the ledger row is readable
---

# Story 10.7: The Sun Lights The Valley

Status: backlog

**Created 2026-09-03 out of 10.4's vehicle sitting**, on Wolf's instruction, so the finding is not
lost. **This story has NOT had a context-filling pass** — the evidence below is complete and
measured, but the open questions in "What Wolf must rule" are unanswered and the ACs are provisional
until he does. Do not dev it from this file alone.

**RUNS BEFORE 10.5.** See "Why this is before the dwarves". The board's key is placed above
10.5 deliberately, because this board's next-story rule reads top to bottom and a prose ruling
alone has silently lost to numeric order twice on this project.

## Story

As the boss,
I want the sun to actually light the valley,
so that every look judgement I make from here on is made under the lighting the game ships with,
instead of the ambient-only scene every judgement so far was made under.

## The defect, measured

`aurora_light_transform()` (`crates/gui/src/atmosphere.rs:209`) places the only `DirectionalLight`
at `aurora_core()`. That helper's height is the **midpoint of the aurora curtain**:

```rust
// crates/gui/src/atmosphere.rs:41-43, 67-71
pub const AURORA_BOTTOM: f32 = -162.0;
pub const AURORA_TOP: f32 = 45.0;
// aurora_core() y = (AURORA_BOTTOM + AURORA_TOP) * 0.5  ==  -58.5
```

**Y = -58.5, while the terrain surface is `CAMP_SURFACE_Y = 9.0`.** The sun sits **67.5 units
beneath the world, shining upward at it.** No visible surface receives it, so it contributes
nothing — and shadows of a light that lights nothing are invisible, which is the symptom Wolf
reported on the vehicle: *"trees don't really generate shadows."*

The line reads correctly at a glance. `aurora_core()` really is the centre of the aurora curtain,
and the curtain really does hang from +45 down to -162. **Using the curtain's centre as the sun's
position is the error** — the curtain is a decorative ring, and its centre is underground.

### The evidence

Headless, lavapipe, `--subdiv 1`, boot framing, `--frames 160`, 1280x720. The instrument is the
frame's **luminance distribution**, not a pixel diff — see the trap on instruments below.

| build | mean luminance | dark (<40) | shade-band (40-89) |
|---|---:|---:|---:|
| shipped, run a | 87.894 | 161,492 | 223,502 |
| shipped, run b — **the noise floor** | 87.973 | 161,495 | 223,412 |
| directional `shadow_maps_enabled: false` | 87.906 | 161,493 | 223,343 |
| `CascadeShadowConfig` max distance 150 -> 500 | 87.865 | 161,489 | 223,492 |
| directional `illuminance: 0.0` — **sun deleted** | 87.815 | 161,489 | 223,560 |
| **sun lifted to Y = 200** | **101.188** | **160,432** | **198,034** |

Two identical builds differ by **0.08 mean and 3 dark pixels in 921,600**. Deleting the sun
entirely moves *less than that*. Lifting it moves the mean by **13.3 — about 170x the noise** — and
empties **25,468** pixels out of the shade band.

## Why this is before the dwarves

10.5 puts the first authored dwarves in front of Wolf for look judgement. **Judging them under a
scene with no sun repeats exactly the failure 10.4 exposed**: candidate D was approved against a
bench frame that differed from what the client drew, and the fix was to make the two agree before
asking for judgement. Same shape here, one level up — approve dwarf assets under ambient-only
lighting and every one of those judgements is provisional until the sun is raised.

## What Wolf must rule

1. **Where the sun goes.** The probe used Y = 200 to prove the mechanism; it is not a proposal.
   Height and angle are a look decision and belong on the bench with artifacts, like any other.
2. **Whether the aurora and the sun stay coupled at all.** They are one entity's transform today.
   The aurora is a decorative curtain; the sun is the key light. They may want separating.
3. **What gets re-judged afterwards.** See the trap below — this moves the ground under several
   settled decisions, and which of them are re-opened is his call, not the dev's.

## Acceptance Criteria (PROVISIONAL — pending the rulings above)

1. `scripts/gate.sh` (the **full** tier) is green, and the diff is confined to this story's own
   commit range from `baseline_commit`.
2. **The sun measurably lights the valley.** The luminance instrument above shows the change at
   **>= 10x the same-build noise floor**, with the noise floor re-measured on this story's own
   build rather than quoted from this file. A figure without its noise floor beside it is not
   evidence — this project has published a delta inside its own noise once already (10.4 AC5).
3. **A guard exists that fails when the sun goes back under the map.** The defect survived because
   nothing could see it: the config sites all read correct. An assertion on the light's world-space
   height against the terrain surface is the cheap shape; a rendered-luminance check is the honest
   one. **Whichever is chosen, it must be shown to fail** by re-applying the shipped transform.
4. **`NEAR_WHITE_AREA_CEILING` is re-measured, not inherited.** It was calibrated on `boot7.png`, a
   frame rendered with the sun off, and 10.4's vehicle sitting already reads 2.2071 % against its
   1.5630 % bar. Raising the sun will move it again. The story either re-calibrates it with its
   evidence or states in one line why the old value still holds. **Do not raise it to clear a
   panic.**
5. **UX-DR22 both halves.** A bench artifact approved by Wolf before the client change, and the
   built result viewed live on the vehicle against that artifact.
6. Mutation rows >= 3, killed, per `mutations/`'s existing format.

## Traps carried in from 10.4

- **Every look decision so far was made with the sun off.** 9.1's blow-out work, 9.4's tree
  colours, 10.3's rules of the look, 10.4's tree judgement, and the near-white ceiling itself.
  Raising the sun moves mean luminance ~15 %. **This does not invalidate those decisions**, but it
  moves the ground they stand on, and any of them may read differently afterwards. Expect to
  re-open the ceiling; do not quietly re-tune anything else without Wolf.
- **A persuasive wrong cause was already falsified here — do not re-find it.** Bevy's never-set
  `CascadeShadowConfig { maximum_distance: 150.0 }` is real, latent, and 150 render units is 150
  **cells** at this project's one-unit-per-cell scale (`crates/gui/src/transform.rs:4`). It fits
  the symptom perfectly and **it is not the cause**: setting it to 500 changed the frame by less
  than noise (row 4 above). Leave it alone until something needs it.
- **Ambient cannot be judged yet.** `ambient_brightness = 4_500` against
  `directional_illuminance = 22_000` (`crates/gui/src/appearance.rs:45,48`) currently has ambient
  doing nearly all the work. Its balance only becomes a real question once the sun is above ground.
- **Use the right instrument.** A per-pixel diff CANNOT resolve this: moving dwarves give a
  38,989-pixel noise floor at delta>=4, larger than the signal. The **luminance distribution** over
  the same captures drops the noise to 0.08 mean and 3 dark pixels. When a diff is swamped, change
  the statistic, not the sample size.
- **Pausing the world to kill that noise does not work here.** The capture's motion health floor
  panics *before* the screenshot when ticks stop, so a paused run writes no PNG at all. Measured
  2026-09-03.
- **`--at-tick` is unusable on both venues** — its floor counts observed ticks, and software
  rendering observes about a third of them while a fast GPU queues them into one frame.

## References

- **`_bmad-output/implementation-artifacts/10-7-signoff/`** — the six captures behind the table
  above, one per row, plus `lumstats.py` (the instrument that found this) and `pixel_diff.py` (the
  one that could not). Committed because dwarf motion makes them unreproducible.
- `_bmad-output/implementation-artifacts/deferred-work.md` — "Found while closing 10.4: THE SUN IS
  UNDER THE MAP (2026-09-03)", the full record with the falsified candidate.
- `_bmad-output/implementation-artifacts/10-4-signoff/task-6-vehicle-runbook.md` — the sitting that
  produced the observation, and the card shape this story should reuse.
- `crates/gui/src/atmosphere.rs:41-43, 67-71, 209` — the constants and the transform.
- `crates/gui/src/ingest.rs:851-862` — `setup_night_lighting`, the only `DirectionalLight`.

## Change Log

| Date | Change |
|---|---|
| 2026-09-03 | Story created out of 10.4's vehicle sitting, on Wolf's instruction ("write sun story so we don't forget it"). Evidence complete and measured; rulings and context pass outstanding. |
