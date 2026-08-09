# Reconciliation — reference image vs. PRD Visual Target

**Input:** `docs/17d7215b-6c05-4286-b3bb-56592ca617ec.jpg` — close-framed concept
render: dwarven mining camp diorama, night, torches, blue mine-entrance glow,
snow, ice pools.
**Against:** `prd.md` §"Visual Target & Game Feel" (+ addendum where relevant).
**Standing rule (restated):** the image is guidance, not an acceptance bar; the
project is procedural-first and the PRD says so. This document asks only: does
the PRD's *text* name the qualities the image actually carries?

---

## 1. What the image communicates (visual-design reading)

### Framing & composition
- High three-quarter view looking *down into* a sunken excavation — the camp sits
  in a dug-out pit one level below the surrounding surface, so verticality and
  cliff edges are visible without any slicing. This is the **working/close
  register** of the PRD's zoom continuum; no vista, no visible aurora in frame.
- The composition is a spiral into the warm center: dark blurred rim → snowy
  surface → pit walls → lit dwarves → the glowing mine mouth as the terminal
  focal point (upper right, deepest point of the scene).
- Strong **miniature/diorama object** read: shallow depth of field (tilt-shift
  blur on the frame edges), and the terrain dissolving into dark haze/void at
  the borders — the world reads as a lit object floating in darkness, not as a
  cropped slice of infinite terrain.

### Palette & values
- Global key: deep desaturated blue-cyan, very low-key overall. **Snow at night
  is a midtone blue-grey, never white** — the brightest values in the frame are
  reserved for emissive sources (torch flames, lantern, the entrance glow) and
  their immediate falloff.
- Warm accents (orange, high saturation) occupy a tiny fraction of frame area
  but win the eye — the cold/warm ratio is extreme, maybe 95/5 by area.
- Ice/water pools are the darkest terrain values: near-black glossy blue with
  faint specular sparkle, punching value holes in the snow field.

### Light behavior
- Warm sources have **short, tight falloff** — intimate pools a few blocks wide
  that tint adjacent voxels and rim-light nearby dwarves; darkness returns fast.
- A soft cool ambient (moon/sky) shapes the snow from above; background haze
  ("air") separates near from far.
- Second, *cold* emissive accent: the blue-teal glow spilling from the mine
  entrance, sourced from visible crystals inside (a **recorded exclusion** —
  see §4).
- Light snowfall is visible against dark backgrounds.

### Material feel
- Chunky voxel geometry with sub-voxel garnish: loose rubble, chipped stone
  fragments (some mid-air off a pickaxe swing), small props (anvil, crate).
- **Snow behaves as a cap layer**: it sits on the *top faces* of stone blocks,
  ledges, the mine lintel, and loads the pine branches; cut faces and cliff
  sides read as dark stratified stone. Every material has a "snowed top / bare
  side" duality.
- Matte snow vs. glossy ice vs. rough stone — three distinct surface responses
  under the same light.

### Readability of figures
- Dwarves read at small size purely by **silhouette + warm rim light**: bearded,
  stocky, distinct poses (two mid-swing at the dig face, one at the
  anvil/crate, one carrying a lantern). Warm skin/leather tones make the
  figures themselves warm objects, not just warm-lit — consistent with the
  PRD's "they are the warm thing in the cold."
- No UI, no markers, no outlines — attention is carried entirely by light and
  color temperature.

---

## 2. Check against the PRD's Visual Target, line by line

| Image quality | PRD coverage | Verdict |
| --- | --- | --- |
| Look *down into* a place, isometric diorama, orbitable | "The view", bullet 1; FR31 | Covered |
| Close/working register: individual dwarves and blocks readable | "The view" bullet 2; anti-req "Cluttered" | Covered |
| Verticality, dug-down camp, underground legible | "The view" bullet 3; FR33; addendum z-level section | Covered |
| Dark blue night world — snow, ice, stone | "The light", bullet 1 | Covered (named explicitly) |
| Small warm pools puncturing the cold; extreme cold/warm area ratio | "pockets of warm light"; Vision paragraph | Covered — "pockets" carries the smallness |
| Eye lands on the camp via temperature contrast, no UI marker | "The light", bullet 2 | Covered |
| Warm sources exist in-world (torches, campfire, lantern) | "The light" bullet 3; FR28, FR29 | Covered |
| Dwarves are themselves warm-toned | Vision: "they are the warm thing" | Covered |
| Depth via light, shadow, and air (haze) | Anti-req "Flat" | Covered |
| Cosmetic snowfall | FR32; NFR5 carve-out | Covered |
| Snow-laden pines | FR27 | Covered |
| Work animates at the dig face; something always moves | Wow beat 2; FR34; anti-req "Lifeless" | Covered (animation); see gap 3 for the *residue* |
| Miniature read: focus falloff, world edge dissolving into void | Not named — "diorama" is used only as a camera/framing word | **Gap 1** |
| Snow as a top-face cap layer; snowed-top/bare-side material duality | Not named anywhere | **Gap 2** |
| Rubble/debris scatter — excavation looks worked | Not named ("work animates" ≠ residue persists) | **Gap 3** |
| Night snow is midtone blue-grey; brightest values reserved for emissives | Not named — "dark blue world" doesn't prevent bright moonlit snow | **Gap 4** |
| Glossy near-black ice pools | "ice" is named in the palette line; gloss level is tuning | Covered enough — story-level tuning, not flagged |
| Blue crystal glow at mine entrance | Out of scope: "no mine crystals" | Recorded exclusion — not flagged |

---

## 3. Gaps — qualities the image carries that the PRD text does not name

Only ones that would change what a dev/artist builds. All four are guidance to
record, not new acceptance bars, per the PRD's own procedural-first framing.

1. **The miniature read is a rendering quality, not just a camera position.**
   The image gets its "diorama" feel from shallow depth of field (tilt-shift
   blur at the frame edges) and from the world dissolving into dark haze at its
   borders — the valley reads as a lit *object in a void*. The PRD uses
   "diorama" only as a framing/orbit description; a dev could satisfy every
   Visual Target line with a sharp, edge-to-edge render and miss this feel
   entirely. Also interacts with the world-edge question: the 128×128 grid's
   boundary wants to fade out, not cut off.
2. **Snow is a cap layer, not a block color.** Top faces of stone, ledges, and
   tree branches carry snow; cut faces and cliff sides read as bare dark stone.
   This top/side material duality is what makes the terrain read as "a cold
   place snow fell on" rather than "white blocks," and it is a worldgen/meshing
   decision (per-face material, snow on exposed top surfaces) that no PRD line
   currently asks for.
3. **Excavation leaves residue.** Loose rubble and stone chips litter the dig
   face and pit floor; fragments fly mid-swing. The PRD's "work animates at the
   dig face" (wow beat 2, FR34) covers the *motion* but not the *evidence* —
   a spotless, debris-free dig site would pass the text and lose the
   worked-earth feel that sells beat 2's "it's alive."
4. **Value discipline: night snow stays midtone; emissives own the brightest
   values.** In the image, snow is blue-grey, and only flames/glow approach
   white. "Dark blue night world" does not prevent the naive failure mode —
   bright moonlit-white snow — which would flatten the warm/cold contrast the
   whole light section depends on. One sentence of value guidance protects the
   wow mechanism.

Suggested landing spot: gaps 1 and 4 as added lines in "The light"/"The view"
(they are boss-visible outcomes, technique-free as required); gaps 2 and 3 as
tech-art/story guidance — addendum or the tech-art-guidelines deliverable.

## 4. Recorded exclusions confirmed present in the image (correctly not flagged)

- **Mine crystals** (source of the blue entrance glow) — PRD Out of scope; the
  *glow* is crystal-derived, so excluded with them.
- **Minecart** — not visible in this crop, and excluded regardless.
- **Built walls** — the mine entrance's dressed-stone arch edges toward "built";
  excluded, recorded.
- Also consistent: no flowing water (pools read as still/frozen), no off-map
  vista, no UI.
