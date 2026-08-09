# Reconciliation — concept image vs. PRD, vista register

Input: `docs/a9d4e72b-b4c3-43f2-8a1c-e25c539fd6c1.jpg` (wide voxel concept render:
frozen valley at night, aurora over mountains, forests, frozen river system,
dwarven camp warm-lit in a dug pit, two further warm-light points at distance).
Checked against `prd.md` (Visual Target, zoom continuum, FR31–FR34, NFR5–NFR6,
out-of-scope) and `addendum.md` (Valheim lesson). Status of the image per the
PRD's own words: guidance and direction, not an acceptance bar.

## What the image communicates about the vista register

### 1. Sky-to-ground ratio and aurora placement
Sky takes roughly the top third of the frame; ground dominates (~2/3). The
aurora is not overhead — it is a set of soft green-teal ribbons hugging the
horizon, **layered behind silhouetted peaks**. That occlusion ordering (terrain
in front of aurora) is what makes the sky read as enormous and far. The aurora
carries *mood*, not area: it is low-intensity, translucent over a dense
starfield, never the brightest thing in frame.

### 2. Aurora and sky as illuminant, not backdrop
The sky light lands on the world. Snowfields and ice pick up teal/cyan
highlights; the frozen river and lakes visibly reflect the sky's hue. The cold
side of the cold/warm contrast is *lit by the sky*, which is why the world
looks cold rather than merely dark. The aurora doubles as the vista's ambient
light source.

### 3. How a small warm camp reads at distance
Warm light at distance reads as **tiny saturated orange points, not lit
areas** — the hilltop lights at far left and the flanked mountain gate at
right are a handful of pixels each, yet the eye finds them instantly, because
orange is the *only* warm hue anywhere in the frame. The lesson for the far
register: the warm/cold read survives arbitrary zoom-out as long as (a) the
cold palette admits zero warm contamination and (b) warm sources stay
point-like and saturated rather than washing out. The image also uses warm
points at three depths (foreground camp, mid hill, far gate) as a depth
ladder — a compositional trick, not a content requirement.

### 4. Atmospheric depth
Depth is carried by air, not geometry: foreground is dark and sharp; each
successive mountain range is lighter, bluer, and less saturated; haze sits
between layers; the horizon glows faintly under the aurora. Classic aerial
perspective over kilometres.

### 5. Ground-plane legibility in the cold palette
At vista distance the ground stays readable because the cold palette is not
one color: white snow, **blue ice (river and lakes, with crack detail)**, dark
pine clusters, dark exposed rock. The winding frozen river is also the frame's
main leading line from horizon to foreground. Remove the ice bodies and the
vista ground collapses toward a monotone white sheet.

## Check against the PRD's zoom-continuum and far-register text

| Image quality | PRD coverage |
| --- | --- |
| Sky/aurora carry the pulled-out frame | **Named.** "pulled out, a vista where the valley, sky, and aurora carry the frame"; FR32 "sky, stars, and aurora carry the far register". |
| Dwarves/camp as warm specks at distance | **Named.** "dwarves become warm specks"; eye lands on camp via warm/cold contrast, not UI (Visual Target). |
| Cold-vs-warm as the organising principle | **Named**, strongly (the wow mechanism section). |
| Warm sources are real in-world emitters | **Named.** FR28/FR29, "things that glow". |
| One continuum, graceful degradation, no mode switch | **Named.** FR31. |
| Atmospheric depth in general | **Partially named.** Anti-requirement "Flat → light, shadow, and air separate near from far"; the addendum's Valheim note ("richness from light and air") points the budget the right way. Neither says what "air" means at diorama scale. |
| Aurora placement/occlusion, sky as illuminant, ice bodies, world-edge treatment | **Not named** — see gaps. |

## Gaps (qualities the image carries that the PRD does not name, and that would change what gets built)

1. **World-edge / surround treatment at vista zoom.** The image is a
   borderless landscape; the PRD's vista is a 128×128 diorama whose edges are
   *in frame* when pulled out. Nothing in the PRD says what the eye meets
   beyond the slab — painted skybox horizon, void, fog skirt, edge-mountain
   worldgen bias. This is the single biggest determinant of whether the far
   register looks like the image's mood or like a cropped chunk floating in
   space, and it drives concrete work (skybox design, or worldgen shaping high
   terrain toward the map rim). NFR5's atmosphere carve-out would permit a
   painted-horizon skybox, but no FR or Visual Target line asks for one.

2. **Aurora/sky as illuminant, not just backdrop.** The PRD's aurora only
   "carries the far register" — content of the sky. In the image it also
   *lights the world*: teal sky-light on snow, sky reflections on ice. Whether
   the sky contributes to ambient lighting is an architecture-shaping decision
   (a skybox texture vs. an environment light), and it is what keeps the cold
   half of the contrast from reading as flat darkness — the exact "Flat"
   anti-requirement.

3. **Cold-palette ground variation — frozen water bodies.** The vista's
   ground-plane legibility leans on blue ice (river, lakes) breaking the white
   snowfield; the frozen river is also the composition's leading line. The PRD
   guarantees only trees (FR27) and shrugs at ice: "Frozen river terrain, *if
   worldgen ever makes one*, is just ice material." As written, a
   snow-plus-trees-only worldgen satisfies every FR and yields a monotone
   vista the image warns against. Either worldgen commits to frozen water
   bodies (a small FR27-sized item) or the PRD should state on the record that
   the vista ground will be snow/trees/rock only.

4. **The aurora's compositional register — low, horizon-hugging, occluded.**
   FR32 names the aurora but not where it sits. The image's aurora works
   because it is low and partially behind terrain silhouettes; an overhead or
   full-sky aurora is an equally valid reading of the PRD text and looks
   completely different. With no off-map peaks to occlude it, achieving the
   image's register means either the skybox paints the silhouettes (gap 1) or
   the aurora is deliberately placed low relative to the diorama. One
   sentence of guidance would prevent a sign-off-artifact round-trip; absent
   that, the sign-off gate is the only thing standing between the two
   readings.

Consciously *not* flagged: the image's second outpost, mountain gate, and
distant ranges as content — the PRD excludes off-map content explicitly and on
purpose, and the multi-depth warm-point ladder cannot be replicated with one
camp on a 128×128 map. That is a known, accepted divergence, not a gap.

## Can 128×128×32 deliver this register, and is the PRD honest about it?

Mostly honest, with one silence. A 128×128 world (~a hundred-odd metres of
terrain) cannot produce the image's kilometre-scale aerial perspective or its
horizon; pulled out, the player sees a *model* — a diorama in a nightscape —
not a landscape. The PRD embraces exactly that framing ("you look *down into*
a place, from outside"), excludes off-map peaks and the second outpost on the
record, and calls the reference guidance-not-bar. That is the honest position.

The silence: the Visual Target's vista sentence ("the valley, sky, and aurora
carry the frame") borrows the image's landscape language without naming the
translation cost — at vista zoom the world's *edges* and the sky *behind and
below them* are what actually carry the frame, and the PRD never mentions the
edge (gap 1). The register is deliverable at 128×128 — a beautifully lit
miniature against a deep sky is a real and achievable look, arguably a better
fit for an orbitable diorama than fake vastness — but it is a *different
picture from the reference*, and the one sentence acknowledging that (and
assigning the surround/edge decision to a story or the architecture pass) is
missing. Recommended disposition: fold gaps 1–4 into the Visual Target /
FR32 as guidance lines, or explicitly route them to the per-story sign-off
artifacts so they are decided on the record rather than defaulted in code.
