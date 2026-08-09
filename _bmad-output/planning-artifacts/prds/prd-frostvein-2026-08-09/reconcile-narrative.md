# Reconciliation — `docs/narrative.md` vs PRD + Addendum (M2, 2026-08-09)

Source: `/workspace/projects/frostvein/docs/narrative.md` — Wolf's first-person
cold-boot narration. PRD frames it correctly as "guidance and direction, not
acceptance bars" (Vision), so the bar here is: every *idea* the source
contributes is either captured, or its exclusion is recorded on the record.

## 1. What the source contributes (item-by-item verdicts)

### §1 Framing the Screenshot

| Source idea | Verdict | Where |
| --- | --- | --- |
| Isometric diorama, look down into a frozen mountain valley | **Captured** | Vision; Visual Target "The view"; FR31 |
| Stylized voxel art | **Captured** | Wow beat 1 "voxel world"; addendum Valheim note ("cheap stylized geometry") |
| Dramatic lighting: deep cold night blues vs pockets of warm orange | **Captured, faithfully** | Vision; "The light (the wow mechanism)" — the warm/cold organising principle is the narrative's exact mechanism, promoted to structure |
| Outpost on a **raised, blocky plateau** | **SILENTLY DROPPED** | Nowhere in scope, out-of-scope, or worldgen FRs. Not on the owner's conscious-exclusion list either. See Gap G2 |
| Fortified with rough stone walls | **Exclusion recorded** | Scope shape Out + "Out of scope" ("built walls... no construction of any kind") |
| Flickering torches, hive of activity | **Captured** | FR28 (torches + campfire), FR34 ("static lights flicker", dwarves work) |
| **Six** dwarves, four actively mining | **SILENTLY DROPPED** | Dwarf count appears nowhere in PRD or addendum. See Gap G1 |
| Mine entrance glowing with **eerie blue-green crystals** | **Exclusion recorded** | "No... mine crystals" (twice). Note: this also removes the narrative's only *cool-colored* light accent; PRD's light model is purely warm-vs-cold — consistent, but worth knowing it was a simplification |
| Small minecart filled with stone | **Exclusion recorded** | "No minecarts..." (Scope shape Out + Out of scope) |
| Heavy snow covering blocky pine trees | **Captured (trees) / subsumed (snow)** | FR27 trees; snow/ice/stone named in "The light". Snow-on-trees texture is worldgen/tech-art detail — acceptable loss at PRD altitude |
| Winding frozen river + fractured ice expanse | **Exclusion recorded, with nuance kept** | "No flowing water; no fluids. Frozen river terrain, if worldgen ever makes one, is just ice material" — a good record: it excludes the *simulation* while leaving the *look* possible. Fractured-ice expanse reads as covered by the same sentence |
| Sprawling dense pine forest on hills | **Captured** | FR27 + `[ASSUMPTION]` density is worldgen tuning |
| Massive snow-capped mountains receding into the distance | **Partially captured / tension unrecorded** | See Gap G3 |
| Second smaller outpost on a distant ridge | **Exclusion recorded** | "No off-map anything... no second outpost" |
| Night sky: dark blue, stars, sweeping green-and-blue aurora across the entire horizon, backlighting the peaks | **Captured (sky/stars/aurora)** — FR32, wow beat 1. Aurora *color* (green/blue) and *backlighting the peaks* composition not stated; colors are fine as guidance-doc detail, the peak-backlighting is part of Gap G3 |

### §2 Where My Eye Lands First

| Source idea | Verdict | Where |
| --- | --- | --- |
| Eye lands on the encampment first | **Captured verbatim in spirit** | "The eye lands on the dwarven encampment first" |
| ...purely because of lighting and contrast ("like a beacon"), not markers | **Captured, including the causal claim** | "...because of the warm/cold contrast, not because of a UI marker" — the PRD kept the *why*, which is the hard part to keep |
| Warm light splashing onto moving figures / texture of stone | **Captured** | "Warm light sources exist *in the world*... contrast is real, not painted on" |
| "Alive and purposeful" vs "vast, cold emptiness"; "massive, indifferent frozen world" | **Captured, phrasing preserved** | Vision: "a cold, dark blue, **indifferent** world... your eye is pulled to the dwarves because they are the warm thing in the cold" — the framing word "indifferent" survived FR-structuring intact |

### §3 What My Hands Are Doing

| Source idea | Verdict | Where |
| --- | --- | --- |
| Mouse-only idle camera rotation; left hand off keyboard | **Captured (mechanism)** | "orbit it by hand" (Vision, FR31). The mouse-orbit control is implied, not stated — acceptable; addendum's z-slice section confirms mouse-first thinking |
| Rotating "to see how the lighting interacts with the voxel geometry" | **Subsumed** | Implied by real in-world light sources (light section) + orbit; no FR forces lighting to respond dynamically to view, but any real implementation of FR28/FR31/FR32 gives this |
| The beat of *pure appreciation before management begins* — the boot state rewards stillness | **Captured** | This is exactly wow beat 1 + the counter-metric "first boot-frame wow — world, light, aurora, **no input needed**". The "no input needed" phrase shows the stillness idea was understood, not just the pretty frame |

### §4 The "Wow" Moment

| Source idea | Verdict | Where |
| --- | --- | --- |
| Beat 1: immediate aesthetic hit (voxel + lighting + aurora) | **Captured** | "The two wow beats" #1, same three ingredients |
| Beat 2 at ~30s: realisation it's *alive* | **Captured, including the timing** | Beat #2 "~Thirty seconds in"; success criterion 1 "both wow beats in one sitting" |
| Static image → functioning simulation transition | **Captured** | "The moment a beautiful still image becomes a running simulation" |
| Micro-beats: torch flicker, pickaxes actually swinging, dwarf picks up an item and carries it | **Captured** | FR34: dwarves "work at the dig face", "carried lanterns move", "static lights flicker"; beat 2: "a dwarf picks something up and carries it". (Destination retargeted from minecart to stockpile — correct given the recorded minecart exclusion) |
| "Daemon is ticking" — aliveness driven by the real backend, not client animation | **Captured, strengthened** | FR34 "driven only by real sim state over the wire"; NFR5 no-drift. PRD made this *harder* than the narrative asked — good |
| "That's the magic" | **Captured** | "This beat is the magic; a client that only achieves beat 1 has failed the milestone" — the priority ordering (beat 2 > beat 1) is the narrative's own emphasis, preserved |

### §5 The 4.1a judgement

Six words — ugly, flat, cluttered, confusing, lifeless, camera not usable —
**captured completely** as the anti-requirements table, each inverted into a
bar, and re-used in success criterion 3. Nothing lost; "camera angle was not
usable" became "you can always reach the angle you want, and never lose the
fortress", which is a fair inversion.

## 2. Conscious-exclusion audit (owner scope decision)

Verifying each named exclusion is recorded somewhere, not silent:

| Exclusion | Recorded? | Where |
| --- | --- | --- |
| Minecart | Yes | Scope shape Out; Out of scope bullet 1 |
| Built walls | Yes | Same two places |
| Crystals | Yes | Same two places ("mine crystals") |
| River (flowing water) | Yes | Out of scope bullet 2, with the ice-material nuance recorded |
| Second outpost | Yes | Scope shape Out ("off-map anything"); Out of scope bullet 3 |
| **Six dwarves vs five** | **NO — silent** | Not in Scope shape, Out of scope, F10, or the addendum. See G1 |

## 3. Gaps

### G1 — Dwarf count exclusion is silent (the one failed exclusion audit)

The narrative shows **six** dwarves; the sim presumably spawns five. This was
per the owner a conscious exclusion, but neither PRD nor addendum records it
anywhere — it is the only item on the conscious-exclusion list that happened
silently. Under the PRD's own rule ("silence is not permission") it needs one
line, e.g. in Out of scope or the Baseline bullet: dwarf count stays at
today's worldgen value; the narrative's six is not a requirement.

### G2 — The raised plateau / elevated outpost terrain was silently dropped

The narrative's outpost sits "on a raised, blocky plateau" — the encampment is
*elevated above* the valley floor, which is part of why the foreground
composition works (warm cluster raised against the dark midground). This is
terrain shape, not construction, so the "no built walls/construction" record
does not cover it, and it is not on the conscious-exclusion list. Nothing says
whether today's seeded worldgen produces anything like it or whether the
Bevy-client stories should care. Either fold it into the FR27-era worldgen
tuning assumption, or record it as out.

### G3 — The background-mountain composition has no owner, and its tension with "no off-map anything" is unrecorded

The narrative's vista is framed by "massive, rugged, snow-capped mountains"
receding into the distance, with the aurora "backlighting the mountain peaks".
The PRD:

- out-scopes "distant peaks beyond the world grid" (recorded, fine), but
- describes the far register as "valley, sky, and aurora carry the frame" —
  **mountains are absent from the vista description**, and
- FR33 ("slice into the mountain") implies in-grid mountains exist.

So the narrative's signature horizon silhouette — peaks against aurora — is
achievable only from in-grid terrain at 128×128×32, and no line says whether
the vista register is expected to deliver it, deliver a reduced version, or
drop it. This is precisely the FR24 defect shape the sign-off gate exists for
(a vista story could be meetable, implemented, and not what Wolf pictured).
One sentence in "The view" or the vista FR31 would close it: either "the
valley's own high terrain gives the vista a skyline" or "the horizon is sky
and aurora only; the narrative's distant crags are out with the off-map
scope."

### Minor / accepted losses (no action needed, listed for completeness)

- Aurora color ("vibrant green and blue") and "stretches across the entire
  horizon" — lives in the referenced guidance docs; acceptable at PRD
  altitude.
- Snow lying *on* the trees; "heavy" snowfall look — tech-art/worldgen
  detail.
- The eerie blue-green (cool) glow accent disappears with the crystals —
  consistent consequence of a recorded exclusion, but the light model
  narrowing to a strict warm/cold binary is a real simplification of the
  narrative's palette; worth a sentence in the tech-art-guidelines
  deliverable, not the PRD.
- "Fractured ice expanse" — reads as covered by the frozen-river/ice-material
  sentence.
- "Mine entrance" as a distinct visual feature — subsumed by dig
  designations + FR33 underground legibility.

## 4. Overall verdict

The PRD is an unusually faithful qualitative capture: the warm/cold causal
mechanism, "indifferent", the two-beat wow structure with the 30-second
timing, the static-image-to-simulation transition, the beat-2-is-the-magic
priority, the no-input stillness of the boot moment, and all six 4.1a words
all survived FR-structuring. Five of the six owner exclusions are properly on
the record. The gaps are three: one silent exclusion (dwarf count), one
silently dropped terrain idea (plateau), and one unowned composition question
(vista mountains vs the off-map exclusion).
