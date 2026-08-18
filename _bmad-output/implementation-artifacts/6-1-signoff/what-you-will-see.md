# Story 6.1 sign-off artifact — "here is what you will see" (UX-DR22 opening half)

> **AMENDMENT 2026-08-18 — read this before the rest of the file.** Two things below are
> superseded by Wolf's rulings at the live viewing. **(1) The named dig site moved** from
> `[58,68,9]`–`[64,69,9]` to `[55,62,9]`–`[56,65,9]`. The original straddled a slope; slope tiles
> are `Tile::Ramp` and are not diggable, so four of them stood as a contiguous wall through the
> middle of the finished excavation — Wolf saw it live and was right. The replacement is 2x4, all
> eight tiles solid, all sky-exposed, unoccluded and in frame, and was re-verified live end to end
> (8 dug in 52 ticks, 8 items, nothing left standing). **(2) The flicker amplitude was raised** from
> torch 0.07 / campfire 0.11 to **0.30 / 0.40**: the mechanism ran correctly all along, but at ~0.1
> stop it read as static to the eye. The before/after PNGs in this directory are of the ORIGINAL
> site and are kept as the record of what was actually approved on 2026-08-17; they are no longer
> the comparison baseline for the dig. Everything else here still stands.

**Status: PART (c) RULED BY WOLF 2026-08-16. PART (a) TAKEN 2026-08-17 and measured — it produced
a finding, now line 8 of part (c). AWAITING WOLF'S APPROVAL of the artifact as a whole. Until that
approval, no implementation commit and no Codex handoff (AC1).**

**Wolf's rulings, 2026-08-16:**

1. **The carried stone: sign wow beat 2 WITHOUT it** (option A below). No sim story is spun, no
   wire change rides on 6.1, and UX-DR14's carried-stone clause is formally not delivered in
   M2 — recorded here rather than blurred. The clause is unobservable in this scenario either
   way, since 6.1 places no stockpile and therefore creates no haul job.
2. **The capture pair is being taken, not skipped** — the fallback was offered and declined, so
   the closing sign-off will be a comparison against two real images of our own world rather
   than against memory.

The other six lines of part (c) were stated and drew no objection.

This is wow beat 2 — the beat the PRD calls the magic. Beat 1 (5.4) sold a *still* image. This
story's whole claim is that the still image starts *running*. A still frame therefore cannot
carry the bar on its own, so this artifact is three parts: a real before/after capture pair from
our own renderer, a written list of the four things 6.1 adds on top of that pair, and an explicit
list of what you will **not** see.

Part (c) is not decoration. 5.4's artifact drew snow-laden spruce sprites the renderer was never
tasked to produce, and the mismatch only surfaced at your live viewing
(`deferred-work.md:635-642`). The rule that came out of it: **an artifact must not substitute
anything the renderer does not itself produce.** Every claim below is traced to shipped code or
to a measurement taken on the shipped seed.

---

## Part (a) — the before/after capture pair

**TAKEN 2026-08-17 by Wolf on gingerspice** off the shipped 5.4 binary, no new code:
`6-1-before.png` and `6-1-after.png`.

### The finding the pair produced — read this before judging the pair

**Wolf's first reaction was "did not see the difference", and he was right.** Measured rather than
argued:

| | |
|---|---|
| Dig-site window, computed from `CameraRig::project_world_point` | `u 0.492–0.541`, `v 0.689–0.747` → **64×43 px** |
| That as a share of the 1280×720 frame | **0.30%** |
| Pixels differing between before and after (channel sum > 30) | **2,255 = 0.245% of the frame** |
| Of those, falling inside the dig-site window | **1,625 — 72%** |

So the dig worked exactly as specified: the designation landed, all 8 tiles went empty, the stone
items rendered, and **72% of every changed pixel sits inside the window the camera math predicted**
— the remaining ~630 are snowfall and aurora. The change is real, correct, and located where it
was supposed to be. It is simply **64×43 pixels in a 1280×720 frame**, which no eye can find in an
A/B flip.

**`6-1-digsite-inset.png` is the pair made legible** — both full frames with the dig-site window
marked, plus 7× nearest-neighbour crops of that window side by side. At 7× the pale shelf is
visibly cut away into a dark trench and a stone item sits at lower centre.

**This is a finding about the artifact and the framing, not about the renderer.** Its consequence
is recorded as line 8 of part (c): the dig face does not read at the boot vista, it reads at
working zoom — which is where AC13's fps reading is taken anyway. The wide pair remains the record
of the composition; the inset is what carries the dig claim. This is also a live preview of what
AC15 exists for: a whole-frame comparison here is satisfied by snowfall alone, and only the
windowed comparison says anything true.

### How the pair was produced

It needed **no new code**. The dig, its rubble and the boot framing all work today, so the pair
came off the **already-shipped 5.4 binary** at
`target/x86_64-pc-windows-gnu/release/gui.exe` (built 2026-08-16 08:30).

```bash
# in WSL
./target/debug/simd 7451

# on the Windows side — the "before": camp at rest, no dig, untouched valley
gui.exe 7451 --capture 6-1-before.png --frames 600

# in WSL — designate the named dig site [58,68,9]-[64,69,9] from a TUI client
./target/debug/tui 7451 --z 9 --frames 3 \
  --key d,h,h,h,h,h,h,j,j,j,j,enter,l,l,l,l,l,l,j,enter

# on the Windows side — the "after": all 8 tiles dug, 8 stone items standing at the site
gui.exe 7451 --capture 6-1-after.png --frames 600
```

Store both in this directory. Measured at story-creation on the shipped seed: the designation
reaches the wire ~2 ticks after the TUI command, the first dwarf enters `Work` ~24 ticks later,
and **all 8 tiles are dug within 52 ticks (~5 s)** with up to 3 dwarves working at once — so a
600-frame "after" run taken any time after the dig has settled shows the finished site.

**Fallback, if no vehicle session is convenient:** the story permits approving this written
document on its own, recording that the pair was skipped and why. What you lose by taking the
fallback is the *baseline* half of the comparison — at the closing sign-off you would be judging
the running client against memory rather than against two images of the same world. Given the
pair costs one `simd`, one TUI command and two already-built binaries, running it is the cheaper
side of that trade. **Your call.**

---

## Part (b) — the four things this story adds on top of that pair

Everything here is client-side presentation. Nothing below changes the simulation, the wire, or
what the TUI shows (AC19: the diff touches `crates/gui`, `docs/` and implementation-artifacts,
and nothing else).

### 1. Dwarves slide between tiles instead of snapping

**The look:** a dwarf crossing from one tile to the next glides across the gap over the tick
interval, so the camp reads as five figures moving through a valley rather than five markers
being re-plotted ten times a second.

The client draws the dwarf strictly *between* the position the wire delivered last tick and the
one it delivered this tick, and it never draws a position the wire has not delivered — no
prediction, no extrapolation, no guessing ahead (AD-15). If the daemon stalls, the dwarf stops at
the last delivered tile and waits; it does not drift onward. Because the blend measures the wire's
own cadence instead of assuming 10 ticks a second, pausing and fast-forwarding come out right for
free: **paused is not still** — the daemon keeps emitting deltas with a frozen tick counter, so
the blend collapses to the identity and the dwarves simply hold position.

**The measured floor this rests on:** with zero commands issued, **47% of ticks (327 of 701)
contain at least one dwarf position change**. There is always something moving.

### 2. Torch and campfire pools breathe

**The look:** the warm pools at the campfire and the four torches swell and ebb continuously —
each emitter on its own phase so the camp never pulses in unison, and the fire reading
differently from the torches. A slow breath, not a strobe and not a flame animation.

The flicker is a pure function of the emitter's simulation id and the client's elapsed seconds —
no randomness, no wire input, no sim meaning — swinging inside a named band around the intensity
5.4's light table already converged on. **The band is deliberately bounded by 5.4's shipped
look:** the capture's warm-pixel floor and ground-luminance checks stay green, so a breath wide
enough to disturb beat 1's frame is too wide by definition.

### 3. Chips of debris at each dug tile

**The look:** where a tile is removed, a small deterministic scatter of chips is left sitting in
the notch — the face reads as *worked* rather than as though the block was cleanly deleted.

These are client-local decoration with no sim meaning: they carry no simulation id, they are the
same on every run for the same tile, and a reconnect or a load wipes and rebuilds them along with
the terrain. That separation is deliberate and is what NFR5 permits — the client may add
presentation, never invent state.

### 4. Stone rubble that stays

**The look:** eight stone cubes sitting at the dug tiles, permanently. The site never returns to
looking untouched.

**This one is already true in the "after" capture** — it is sim-side and shipped: a dig spawns a
stone item at the dug tile, `gui` already draws items as stone cubes, and with **no stockpile
placed nothing ever hauls them away**. It is listed because it is half of what "work leaves
evidence" means, and because the before/after pair is what shows it.

---

## Part (c) — what you will NOT see

**Each line below wants your ruling, not just your reading.** These are the gaps between what
6.1 delivers and what a viewer might reasonably expect, named in advance so none of them arrives
as a surprise at the live viewing. Lines 1–7 were written before the pair was taken; **line 8 was
added afterwards because the pair itself taught it** — which is the gate doing its job at the price
of two screenshots.

| # | Not in this story | Where it lives instead |
|---|---|---|
| 1 | **No lantern light.** `LightKind::Lantern` keeps its table row and stays unused; dwarves carry no light and are not special-cased warm. | 6.2 — and 6.2 is **first on the M2 cut list**, so this may never ship. Epic 6's wow is intact without it. |
| 2 | **No z-slicing.** You see the surface; you cannot cut down into the mountain. | 7.1 |
| 3 | **No mouse, no commands from the Bevy client.** Every order in this story is issued from a TUI client on the same daemon. The Bevy window is a viewer. | 8.x |
| 4 | **Dwarves remain scaled cubes.** No models, no limbs, no walk cycle — a cube slides rigidly across the gap. The motion is smooth; the *thing* moving is still a box. | No story owns dwarf models in M2. Raised explicitly because this is a motion story, and smooth motion tends to draw the eye to what is moving. |
| 5 | **Trees remain wire-true cube stacks**, not the spruce sprites 5.4's approved artifact drew. Your 5.4 ruling stands: full wire-true density, no worldgen change. | Unchanged from 5.4 — listed so the two artifacts are not read against each other. |
| 6 | **Only the light breathes, not the emitter.** The point light's intensity flickers; the campfire's own material does not glow brighter with it — per-entity emissive materials would mean one material handle per emitter. | Deferred; no story owns it. |
| 7 | **A carried stone does not travel with its dwarf.** ✅ **RULED 2026-08-16: beat 2 is signed without it.** UX-DR14's carried-stone clause is formally not delivered in M2. | Sim + wire change; out of scope by AC19. Not scheduled. |
| 8 | **You will NOT see the dig face at the boot vista.** It is 64×43 px — 0.30% of the frame — so the dug tiles, the stone rubble and the debris chips are all sub-legible at the opening framing. **They read at working zoom.** Measured on the 2026-08-17 pair, above. Consequence for the closing sign-off: judge the *dig* at working zoom, and judge the *boot frame* for the composition and the motion. | Inherent to the boot framing, which is 5.4's approved composition and is not being changed here. |

### The raise: UX-DR14's carried stone

UX-DR14's wording includes *"a dwarf picks something up and carries it"*. **That is not
achievable in any client today, and it is not a client problem.**

Verified against shipped `sim-core`:

- The public accessor that feeds the wire, `World::items()`, reports **every** item at its stored
  position, carried or not (`crates/sim-core/src/lib.rs:1462-1471`).
- A stone's position is rewritten in exactly one place — `release_claim`, i.e. **the drop**
  (`:696-707`). While a dwarf carries it, the stone's position stays on its pickup tile.
- So on the wire a carried stone sits still where it was picked up and then **teleports** to
  where it was dropped. The sim itself knows better (`uncarried_stones` excludes carried stones
  from job logic, `:674-687`) — that knowledge simply never reaches the wire.
- The TUI's "carrier" glyph never contradicted this: it only means *a dwarf standing on a tile
  that has an item* (`crates/tui/src/view.rs:240-251`).

Making it visible means a sim change **and** a wire change, which AC19 forbids in this story
(AD-16's sanctioned M2 wire diff was spent at 5.1).

**One fact that materially changes the decision, found while verifying the above:** haul jobs are
derived **only from stockpile tiles** (`crates/sim-core/src/lib.rs:319`, `:260`), and **6.1 places
no stockpile** — that is deliberate, it is what makes the rubble pile up at the face. So in this
story's scenario **no dwarf ever picks a stone up at all.** The carried-stone gap is not merely
invisible here; it does not occur. It cannot be seen to be missing, and no viewer watching the
6.1 dig will notice its absence.

**Your call, and it is a real fork.** ✅ **WOLF RULED (A), 2026-08-16.**

- **(A) Sign beat 2 without it.** ✅ **TAKEN.** UX-DR14's other clauses — the valley visibly living, work
  leaving evidence, light breathing — are all delivered and all measurable. The carried-stone
  clause is unobservable in this scenario either way. *This is my recommendation:* it keeps the
  story gui-only, keeps the wire frozen, and buys nothing visible by breaking either.
- **(B) NOT taken. Spin a separate sim story** that puts carried items on the wire (a stone's position
  following its carrier, or an explicit carrier field) plus a stockpile in the demo so hauling
  actually happens. That is a sim + protocol + client slice of its own. Note the cost honestly:
  it competes for M2 capacity against 6.2, which is already first on the cut list.

---

## Provenance

Every measurement quoted here was taken at story-creation on a **live `simd` on the shipped
seed**, not estimated: designations at tick 46, dwarves in `Work` by tick 70, all 8 tiles empty by
tick 98, 8 stone items still standing at `[58..64, 68..69, 9]` through a 70-second observation;
327 of 701 ticks carrying a dwarf position change with zero commands issued; the site's projected
screen window computed from the shipped `CameraRig` constants at `(0.49,0.70)`–`(0.53,0.73)`,
lower centre of frame, inside the camp's light. The code claims in Part (c) were re-verified
against `crates/sim-core` in this session.
