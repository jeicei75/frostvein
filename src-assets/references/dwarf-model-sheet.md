# Dwarf model sheet — the reference, as numbers

**Why this file exists.** Rounds 1–3 each delivered every *checkable* requirement and missed every
*aesthetic* one. That is not a bad art seat; it is a process that converts tests into compliance and
prose into noise. "Value steps", "detail in the silhouette", "read like the reference" had no test,
so they did not converge. This sheet turns the look into numbers so it can converge the same way the
mechanical clauses already do.

It is the FIRST deliverable of a round, before any geometry: measure the reference, agree the sheet,
then check the model against it.

## The instruments, and what each can honestly measure

**MEASURE FROM `src-assets/references/dwarf-ortho/` — THE ORTHOGRAPHIC SHEET. Everything below
that was measured off the video is PROVISIONAL.** Found 2026-09-11, after four rounds:
`reference-sheet.jpg` in the same directory is the **modelling reference sheet the video was
rendered from**, and it carries orthographic **front, both sides and back** views plus a gear
breakdown and the palette with hex codes. Rounds 1–4 measured `dwarf.mp4` instead — a lit,
perspective, compressed render two generations downstream of it. So:

- **every depth dimension round 4 had to estimate is directly measurable**, and the back it had to
  invent is drawn;
- the ±4 % caveat this sheet carries was never necessary;
- the palette swatches on the reference sheet **are** our approved ten (`Tunic #5F7A6A`), which
  closes "the reference looks warmer" as an artifact of the single dim frame it was asked about;
- **re-measure every proportion below against `dwarf-ortho/front.png` and `side-left.png` before
  trusting it.**

Measure the ART, not the sheet's annotations — at least one is incoherent (its front view labels the
dwarf's own height `0.6x dwarf height`).

**Secondary: `src-assets/references/dwarf-frames/`, not t=10 s.** Added 2026-09-11 after
round 4: every proportion in this sheet was measured from ONE frame, t=10 s, because that is what an
`fps=1` sample handed me — and it is among the worst frames in the file. The video holds 240 frames
including brightly-lit near-front and **near-pure side** views. `f088` is the depth authority (every
Y dimension round 4 had to estimate), `f104` the front authority, `f084` shows the head's **domed,
stepped crown**, and `f140`/`f164` show the **neck and shoulder caps** that t=10 s hides. See that
directory's README. **Round 4's flat side profile and featureless head are downstream of this, not of
its modelling.** The numbers below still stand where they were cross-checked, but re-measure against
`f088`/`f104` before trusting any of them.

**`src-assets/references/dwarf.mp4` — the look authority, and a poor measuring surface.** It is
10 s of h264 in 3.4 MB, lit, in perspective, in a dark mine. Landmark heights read to about **±4 %**
off a gridded frame. Albedo, saturation and silhouette share **cannot** be recovered from it: the
brightest pixels in frame are the torch and the lantern, and every surface value has been through a
lit scene and a view transform. Numbers below tagged `ref≈` are eyeball reads off the t=10 s frame
at a 20 px grid; treat them as ±4 %, not as truth.

**Our own renders — exact.** `render_dwarf.py` puts the figure on flat `#6F7073`, so background
subtraction is trivial and every figure measurement below is exact for the pixels it names. That
asymmetry is deliberate: **the reference sets targets loosely, our render is measured tightly.**

**`src-assets/references/dwarf-contact-sheet.jpg` — the palette and gear authority.** It is what the
approved ten palette cells were read from, and it stays the authority for WHAT gear exists.

## Proportions — the sheet's core

> **SUPERSEDED 2026-09-11 by round 5.** Everything below this box was measured off `dwarf.mp4`, a
> lit, perspective, h264 render two generations downstream of the orthographic sheet. Round 5
> measured `src-assets/references/dwarf-ortho/` instead — 5x nearest-neighbour crops of
> `reference-sheet.jpg`, where every source pixel is a countable 5x5 block — and the table in the
> next section replaces this one. The old numbers are kept because the *reasoning* about the beard
> (a width check, not a length check) still holds and because two of them turned out close.

### Measured off the orthographic views — round 5, and this is the authority

The figure spans **rows 7 (crown) to 147 (sole)** in both `front.png` and `side-left.png`, so
**140 source pixels = 1.00 H** and one source pixel is 8.571 mm on a 1.20 m dwarf. `back.png` is
123 px for the same figure and its pixels are scaled by 140/123 before use. `front.png` column 77 is
the centre line; `side-left.png` column 58 is the depth centre, +Y forward.

**Heights, as a fraction of figure height measured UP FROM THE SOLE** (the video table below measures
down from the crown; they are complements):

| landmark | z / H | source |
|---|---|---|
| crown | 1.000 | front rows 7 |
| crown step 2 / step 1 / main skull top | 0.979 / 0.964 / 0.943 | front rows 10 / 12 / 15 |
| brow | 0.879 | front row 24 |
| eye line | 0.807 | front rows 30–34 |
| ear top / ear bottom | 0.850 / 0.729 | front rows 28 / 45 |
| nose tip (lowest) | 0.750 | front row 42 |
| head + hair mass ends | **0.707** | front row 48, back row 44 — the two agree to 0.2 % |
| shoulder line (top of the sleeve cap) | **0.700** | back row 45 |
| sleeve cuff, bare forearm begins | 0.566 | back row 61 |
| **beard tip** | **0.464** | front row 82 |
| belt top / belt bottom | 0.421 / 0.343 | front rows 88 / 99 |
| pack bottom | ~0.35 | side row 95, back row 90 |
| hand bottom | 0.330 | back row 90 |
| **tunic hem** | **0.207** | front row 118, back row 106 — the two agree to 0.5 % |
| boot cuff top / bottom | 0.164 / 0.107 | front rows 124 / 132 |
| sole | 0.000 | front row 147 |

**Widths, as a fraction of figure height:**

| feature | width / H | source |
|---|---|---|
| head, with hair | **0.336** | front cols 54–100; back 0.341, so the two agree to 1.5 % |
| head, ear to ear | 0.383 | front cols 51–104; back 0.382 |
| crown steps, top down | 0.164 / 0.229 / 0.286 / 0.336 | front rows 7 / 10 / 12 / 15 |
| beard, widest | 0.343 | front rows 44–56 — essentially the head's own width |
| chest, tunic only | 0.317 | back, between the sleeve seams |
| **shoulders, over the sleeve caps** | **0.528** | back cols 26–90 |
| waist / tunic skirt | 0.439 | back cols 32–85; front 0.440 |
| arm span, hands out (the sheet is POSED) | 0.772 | back cols 11–105 |
| stance, boot outer to boot outer | 0.398 | front cols 50–105; back 0.398 |
| one leg / one boot | 0.164 | front, each leg |
| boot cuff | 0.200 | front rows 125–132 |

**Depths, as a fraction of figure height, +Y forward, measured on `side-left.png`:**

| feature | depth / H | source |
|---|---|---|
| head, back to front of the hair | 0.336 | cols 36–83 — the head is very nearly a cube |
| nose tip, past the back of the head | 0.379 | col 89 |
| beard front | 0.357 | col 86 |
| torso | 0.286 | cols 38–78 |
| tunic skirt | 0.300 | cols 37–79 |
| pack, behind the torso back | 0.186 | cols 12–38 |
| shin | 0.164 | cols 47–70 |
| **boot, sole length** | **0.214** | cols 47–76 — the toe projects 0.057 H past the shin front |
| whole figure, pack to nose | 0.550 | cols 12–89 |

**The shoulder-against-head number is the one that matters most**, and it is the one the video could
not give: a 0.528 H shoulder against a 0.336 H head leaves **0.096 H of shoulder outboard of the
skull on each side**. Round 4 measured 0.336 m of head against 0.350 m of torso, left 7 mm of
shoulder, and had nowhere to route the over-shoulder strap the sheet plainly draws. The sheet
settles it: the torso is not the thing that carries the shoulder — the sleeve caps are.

**Two labels on the sheet are decorative and must not be obeyed.** The front view's vertical
dimension reads `0.8x dwarf height` against the dwarf's own height, and `12 Voxels` across the body
would imply a 15-voxel-tall figure the artwork plainly exceeds. The horizontal labels ARE consistent
with the art: the front view including the pickaxe measures 1.079 H against its `1.0x` label, and
each side view measures 0.643 H against its `0.67x`. Measure the art; read the labels as a sanity
check only.

---

**CORRECTED 2026-09-11, and the correction came from the art seat.** The first edition of this table
carried one row reading "head + beard mass ends — `ref≈ 24 %`" against r3's 48.5 %. That row
compared **two different quantities**: 24 % is where the HEAD ends (the shoulder line), and 48.5 %
was a row-majority measure, which is a measure of the beard's **WIDTH**, not its length. The seat
measured the reference itself, got the head at 24.7 % — matching — and the **beard tip at 42.8 %**,
hanging to just above the belt. Re-verified here independently: in the t=10 s frame the olive tunic
does not appear until source y≈323 and the beard mass runs to ~350, with the buckle at ~425. **The
reference's beard is long. r3's fault was that it was WIDE.**

| landmark | reference (±4 %) | r3 measured | verdict |
|---|---|---|---|
| head mass ends (shoulder line) | **24.7 %** | ~25 % | fine — never the problem |
| **beard tip** | **42.8 %** | ~43 % | **fine — do NOT shorten it** |
| **beard width vs head width** | **0.25 m against a 0.34 m head — NARROWER than the head** | crosses the chest | **THE fault** |
| belt / buckle centre | `ref≈ 53–56 %` | ~50 % | close |
| tunic hem | `ref≈ 67 %` | ~62 % | close |
| boot top | `ref≈ 83 %` | ~80 % | close |
| ground | 100 % | 100 % | — |

**So the check is a width check, not a length check.** A beard that reaches the belt is correct and
matches the video; a beard wider than the head is what made r3 read as "a beard with legs" and hid
the tunic, belt and buckle that give the character its colour. Keep the length, take the width in.

Per-tenth breakdown of r3's body band, brown (beard/hair/leather/boots) against tunic:

| band | brown | tunic |
|---|---|---|
| 0–10 % | 99.0 % | 0.0 % |
| 10–20 % | 98.8 % | 1.2 % |
| 20–30 % | 91.2 % | 8.4 % |
| 30–40 % | 99.3 % | 0.5 % |
| 40–50 % | 63.5 % | 36.3 % |
| 50–60 % | 18.7 % | 80.9 % |
| 60–70 % | 57.9 % | 38.6 % |
| 70–80 % | 37.7 % | 15.9 % |
| 80–90 % | 84.2 % | 0.0 % |
| 90–100 % | 99.2 % | 0.4 % |

The tunic owns exactly one band of ten. **Read this as a CHEST-COVERAGE measure and nothing else** —
it answers "does something cover the chest", not "how long is the beard". The target it implies:
the tunic should win most bands between roughly 25 % and 60 % of height, which happens when the
beard is narrow, regardless of how far down it hangs.

Whole-figure colour-family shares, r3, exact: **brown 59.2 %, tunic 22.5 %, grey 11.5 %, skin
6.8 %** of 259,172 figure pixels. The reference's shares are deliberately NOT quoted — they are not
measurable from the video — but it reads with far more tunic and visible skin, and a beard that
does not cross the chest.

## Feature scale

The unit for feature sizes is **1/64 of figure height** (~1.9 cm on a 1.20 m dwarf), which is what
the reference's finest feature measures — its belt buckle is about **5 units** across, its belt
about **2 units** tall. Use it to size gear, not to quantise positions: **there is no lattice**
(see the round-4 brief).

| feature | reference size |
|---|---|
| belt buckle plate | ≈ 5 units across |
| belt band | ≈ 2 units tall |
| finest authored detail | ≈ 1 unit |

## Palette — authority, and one open decision

The **approved ten cells**, read from the contact sheet and signed off, remain the base:
`#E9D2BB` skin, `#5E4632`, `#FFFFFF`, `#5F7A6A` tunic, `#474B41`, `#A9B2AC` metal, `#8B6B50` wood,
`#6B5B49`, `#34271C` hair, `#F0A63C` flame.

r3 expanded these correctly to **23 cells** by adding value steps (`#BAA896`, `#826145`, `#423123`,
`#7DA18C`, `#44584C`, `#63695B`, `#CBD6CE`, `#707572`, `#AF8765`, `#8F7A62`, `#493E32`, `#513C2B`,
`#F7CE94`) — that ask landed and should not be redone.

**RULED 2026-09-11 — Wolf: *"we can keep approved palette right now.. let's see after epic 11 will
we change the palette"*.** The approved hues stand.

**AND A CORRECTION, also from the art seat: "inherit r3's 23 cells" was not reachable.** Two reasons,
both mine:

1. **The 23 cells live on the `dwarf-round-3-brief` branch** — in its candidate GLB and in that
   branch's `dwarf_miner.py`. Round 4's branch came off `main`, so the seat is looking at the
   shipped r2 asset (10 cells) and main's generator, where the 13 value steps do not exist. The
   hexes are listed in this sheet, but the **atlas layout** — which cell sits at which coordinate —
   is not, and that is what a palette actually is.
2. **The contract as ENFORCED allows 16 cells, not 23.** `check_asset.py` fixes the atlas at 64 px
   with 16 px cells, so 16 is the ceiling. r3's atlas used 8 px cells and 23 colours and passed
   anyway — **because the palette clause reads only 4 of them.** The ceiling the seat ran into is an
   artifact of the same broken clause that reported `palette=` with four entries.

So the 16-cell ceiling is real until the clause is fixed, and fixing it is owed work on this side,
not the seat's. Until then: **build on the 10 approved cells, which round-trip exactly, and propose
which of the 13 value steps earn the 6 spare slots as the parts that need them arrive.** The question that prompted this stays on the record because it will come back: the video
reads warmer and more saturated than the approved cells — its tunic is an olive-yellow where
`#5F7A6A` is a desaturated green at saturation 0.22 — and since the video is lit and compressed, the
true albedo is not recoverable from it.

**Why deferring is the right call and not just a delay.** Epic 11 adds ambient occlusion, bloom,
exposure, depth of field and a day/night cycle. Every one of those changes how an albedo reads, so a
palette warmed now would be tuned against a renderer that is about to change underneath it. This
project has already paid that bill once: issue **#75** records that every look constant predating
10.7 was tuned with the sun under the map.

**Revisit trigger:** Epic 11 complete, then re-judge the dwarf's palette under the finished
lighting. Until then a round must not re-derive colours from the video — that is how a palette gets
read off a lit frame and is wrong in silence.

## Budget and LODs — RECORDED FOR LATER, AND EXPLICITLY NOT A CONSTRAINT ON THIS ROUND

**Wolf, 2026-09-11: *"no need to worry that now .. let's keep stretching limits first"*.** So none
of the numbers in this section bind the next round. They are written down because they are derived
and will matter, not because they should shape the model now.

**Why that order is right, and not just the boss's preference:** cost is a mechanical property, and
this process already converges on mechanical asks — r3 hit every one it was given. Likeness is the
half that does not converge. Optimising triangles now would spend the scarce thing (art iteration)
on the abundant one (a decimation pass later). And a box model decimates by *removing boxes*, so the
LOD chain stays cheap to add after the look is settled, which a greedy-meshed voxel body would not.

Derived from our own frame, not quoted from an article:

There is no numeric frame budget on record; NFR6's bar is the adjective "interactive framerates". So
these come from measurement: the terrain at `--subdiv 4` is **~124,000 triangles**, and r3's dwarf
is **11,058**.

**Twenty r3 dwarves would cost 221,000 triangles — nearly twice the entire terrain.** With hundreds
intended, r3 is off by roughly an order of magnitude, and this is a second independent reason not to
build the dwarf as a greedy-meshed voxel body: a voxel mesh cannot be decimated without turning to
mush, while a **box model reduces by removing boxes**, which stays exactly in style.

| LOD | screen height | triangle ceiling | what it is for |
|---|---|---|---|
| LOD0 | > 200 px | **≤ 4,000** | marketing shots, close-ups; a handful visible |
| LOD1 | 40–200 px | **≤ 800** | inspection range; tens visible |
| LOD2 | < 40 px | **≤ 150** | normal play; hundreds visible |

Ceilings, never targets. At 200 LOD2 dwarves that is 30,000 triangles, about a quarter of the
terrain — sane. And note what the current game framing means: **a dwarf is 8.74 px tall at the boot
camera**, measured through the client's own projection oracle, so essentially all gameplay is LOD2.
**The LOD2 silhouette is what dwarves actually look like in play, and deserves as much art attention
as LOD0.**

Beyond hundreds the ladder stops being about triangles: at thousands, LOD2 must become GPU-instanced
and unskinned, because skinning thousands of characters costs CPU first; at tens of thousands, a
2-triangle camera-facing impostor is indistinguishable from the model at 8.74 px. Not this round's
problem, but it is the reason not to spend art effort between LOD0 and the small silhouette.

## Where this ends up: hand-model the PARTS, generate the COMBINATIONS

Wolf's stated ambition, 2026-09-11: *"I want to give own dwarf as a gift for every living people and
wolf an earth"*. Taken seriously, that is a **generation** requirement, not a rendering one — a
dwarf becomes a SEED, and the generator makes it on demand with nothing stored.

The arithmetic says it is reachable and says what shape the system has to be. Discrete parts alone —
say 8 heads x 12 beards x 8 hairs x 4 packs x 3 tools, with 5 skins x 8 tunics x 6 hair colours from
the palette — is about **2.2 million** dwarves. Not eight billion. Continuous dials are what close
the gap: height, girth, beard length, nose size, quantised finely, multiply that by thousands. So
roughly **twelve discrete axes plus four continuous dials** clears eight billion.

**The design consequence, and it is the thing this round could accidentally destroy:** r3 already
implemented the discrete half as `--beard NAME --hair NAME --pose NAME`. Retiring r3's geometry must
not retire its **parameterisation**. The endpoint is not "hand-modelled" versus "procedural" — it is
a **hand-authored parts library** (shaped by eye, which is the half that does not converge on its
own) **assembled procedurally from a seed** (the half that does). The `.blend` holds the parts; a
script picks, scales and recolours them.

Two properties that fall out of that shape for free, which is how you know it is the right one:
every variant shares one mesh set, so they instance; and a dwarf costs eight bytes to store and
name, so "your dwarf" is reproducible anywhere from an identifier.

## Check 4 — SILHOUETTE STEP DENSITY, the measure of "more detail"

Added 2026-09-11 after round 5, because Wolf's *"resolution could be higher with more details"* had
no number behind it and every unnumbered ask in this project has failed to converge.

**The measure:** walk the figure's left and right silhouette edges row by row and count the rows
where the edge MOVES. Normalise by figure height as steps per 100 rows, so renders of different
sizes compare. It needs no palette, no classifier and no lighting — just the silhouette.

| view | reference (from `dwarf-ortho/`) | r5 measured |
|---|---|---|
| side | **10.3** steps / 100 rows | 7.1 |
| front | **11.2** steps / 100 rows | 6.3 |

**So r5 carries about 60 % of the reference's detail density.** That is the honest content of "more
detail", and — critically — **it is NOT an argument for a lattice or a finer grid.** It is an
argument for MORE AND SMALLER BOXES where the sheet shows a step we do not have: the stepped crown,
the brow, the nose, the pack standing off the back, boot layers, strap plates.

**And the sharper finding, which the density figure alone hides: r5's edges are asymmetric.** Side
view L33 / R12 steps; front L13 / R27. The reference is near-symmetric — L33/R36 and L37/R38. **One
side of r5 is a straight slab**, which is exactly what "a flat blocky stick" describes. Both figures
include held props on one edge, so the asymmetry is not purely the body — but a 3:1 ratio is not
props.

**The targets:**

1. **>= 10 silhouette steps per 100 rows** on both the front and the side view.
2. **Neither edge below 70 % of the other** on the same view.

There is no triangle budget this round, and r5 spent 1,116 against an eventual LOD0 ceiling of
4,000 — so the headroom for this exists three times over. Spend it on steps, not on smoothness.

## Check 5 — THE NECK IS BARE IN PROFILE

Added 2026-09-12, on Wolf's *"neck is visible on side view of reference images"*, because round 5
built a neck that no view can see and reported the sheet as having none.

Measured on `dwarf-ortho/side-left.png` and `side-right.png`, which agree to the row: a bare skin
column from **z/H 0.793 down to 0.679** — **0.121 H of visible neck**, up to **0.064 H wide**,
landing on the collar just below the shoulder line (0.700). The hair falls **either side of it**
rather than over it. From the front the beard covers it, which is why four rounds of front-view
measurement missed it.

**The target:** on our own `dwarf-flat-side-left.png` and `dwarf-flat-side-right.png`, a run of skin
pixels at least **0.08 H tall** must be visible between the hair and the collar, in **both** side
views. Measure it the way the reference was measured — classify to nearest palette cell, walk the
rows, take the longest contiguous run.

**This is a geometry clause, not a beard clause.** The lever is where the HAIR falls, not how wide
the beard is: the reference keeps a full beard AND a bare neck by parting the hair around the neck.
Do not narrow the beard to chase this.

## What round 4 is checked against

Numeric, on our own render, which is exactly measurable:

1. **The beard is no wider than the head** (reference: 0.25 m beard against a 0.34 m head; r3:
   crossed the chest). This replaces the withdrawn "head + beard ends by 30 %" check, which was
   measuring the wrong thing — **the beard's LENGTH should match the video's 42.8 %, not be cut.**
2. **The tunic family owns at least four of the ten height bands** (r3: one) — a chest-coverage
   consequence of check 1, not an independent target.
3. **Visible skin is at least 10 % of figure pixels** (r3: 6.8 %) — the face has to read.
   **MEASURE IT BY NEAREST PALETTE CELL, NOT BY EXACT COLOUR MATCH.** Corrected 2026-09-11 after
   round 4 recorded this check as a miss at 7.4 %: the "flat" pass is antialiased, so each cell is
   smeared across dozens of near-duplicate values (`#E9D2BB`, `#E9D2BA`, `#EAD2BB`, `#E9D1BA`…) and
   an exact match counts only the pure core of every region. Classifying each figure pixel to its
   nearest cell instead — and discarding the 0.4 % that are background blends — gives **13.9 %**,
   and the check PASSES. The model was never short of skin; the instrument was. Any future share
   measured off a render must cluster to cells the same way.
4. Feature sizes within the table above — **advisory this round, not a gate.**
5. Every mechanical clause of the asset contract, which r3 already proves the seat can hit.

**Deliberately NOT a check this round: the triangle count.** Wolf's call — stretch the look first,
optimise after. If the round comes back at 40,000 triangles and looks like the reference, that is a
success followed by a decimation task, not a failure.

Items 1–3 are the likeness half made checkable. They are not the whole of "looks like the
reference", and they are not meant to be — they are the three biggest measured deltas between r3 and
the thing Wolf keeps pointing at.
