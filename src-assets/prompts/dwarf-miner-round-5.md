# Round 5 — build it from the orthographic sheet

**Start the model from scratch. The reason is that the INPUTS changed, not that round 4 was bad.**

Round 4 was a good round: it hit every mechanical clause, cut the triangle count from 14,398 to
1,214, passed all three of the model sheet's numeric checks (see the correction below), refused to
cut the beard to chase a bad metric, and reported its own limits honestly. But it had to **estimate
every depth dimension and invent the back**, because the only reference anyone had given it was one
frame of a compressed video.

**That was wrong, and the fault was the orchestrator's.** `src-assets/references/reference-sheet.jpg`
is the **modelling reference sheet the video was rendered from**. It carries orthographic **front,
both sides, and back**, a gear breakdown with proportions, and the palette with hex codes. It was in
the same directory the whole time. Four rounds measured the downstream artifact.

So this round rebuilds on the real source. A model whose depths are measured rather than guessed is
a different object from round 4, which is why it is a rebuild and not an edit.

## Run it through — no per-step approval this round

**Wolf, 2026-09-11: *"let's let it work now without asking my comments.. if we are close then we
will have fixing round in the end of modelling.. if we are not close then we will have a new
brief"*.** So:

- **Do not stop and wait at any point.** Work through every part to a complete figure and deliver.
- **Keep saving the `.blend` and writing a progress render at every step anyway.** Those are not the
  gate; they are the crash insurance and the record. Four of six delegated runs in this project have
  been killed by the harness.
- **Still report every conflict, in the report, in writing.** Round 4's step-1 questions found two
  real defects in the input documents. That behaviour is wanted — what is not wanted is it blocking
  on an answer. When an input contradicts the reference, follow the reference, and write down what
  you did and why.
- **A fixing round follows if the result is close**, and a new brief if it is not. So **work
  breadth-first: get every part present and the whole figure standing, and do not gold-plate one
  part.** A complete figure that is 80 % right everywhere can be judged and fixed; three perfect
  parts and a missing leg cannot.

## Step 0 — RE-MEASURE and record it, then keep going

**Measure first and put the table in your report. Do not wait for it to be approved.**

Measure from `src-assets/references/dwarf-ortho/` (5x nearest-neighbour crops of the sheet, so every
source pixel is a countable 5x5 block — read its README first):

- `front.png` — front proportions, the face, **shoulder width against head width**
- `side-left.png` — **every depth dimension**: head and beard projection, torso depth, pack
  projection, boot length forward
- `side-right.png` — cross-check, and the lantern arm
- `back.png` — the back, pack straps and flap
- `gear.png` — pickaxe **0.83x dwarf height**, lantern **0.33x**, from the sheet's own labels

Record a table of proportions as fractions of dwarf height in the report, the way
`src-assets/references/dwarf-model-sheet.md` does — **and treat that file as PROVISIONAL. Every
number in it was measured off the video and is superseded by anything you measure off the
orthographic views.** Correct it rather than obeying it; step 1 of round 4 already proved that
behaviour right, and this time the sheet's author has said so in writing.

**Measure the ART, not the sheet's annotations.** At least one is incoherent — the front view labels
the dwarf's own height `0.6x dwarf height` — and `12 Voxels` across the body would imply a
15-voxel-tall dwarf the artwork plainly exceeds. The drawings are orthographic and trustworthy; the
numbers printed beside them are not.

## The five defects round 4 had, with what the sheet says about each

Wolf's verdict on round 4, in his words: *"it's still lacking form... head is one big block... side
view is kind of flat blocky stick... feet could be a bit longer... maybe we should have shoulders...
also neck is missing"*. Four of those five are **fidelity**, not departures — the sheet has them and
the video hid them:

1. **The head is not a box.** Both side views show a **stepped, domed crown** — it steps in at the
   front and at the back. Within "no rotated boxes" that is reachable by stepping the crown with two
   or three smaller boxes. Round 4's single box is the minimum-effort reading of the reference.
2. **The side profile is not flat.** The sheet's side view is a deep torso with a forward-projecting
   brow, nose and beard. Every one of those depths is now measurable.
3. **There is a neck.** Visible on the sheet between beard and collar. Round 4 seats the head
   directly on the torso.
4. **There are shoulder caps.** The sleeve sits ON the shoulder and the torso is wider at the top
   than at the waist. **Re-measure head width against shoulder width off `front.png` before you
   commit to proportions** — round 4 measured a 0.336 m head against a 0.350 m torso, which left
   7 mm of shoulder and nowhere to route the over-shoulder strap the sheet clearly has. One of those
   two numbers is wrong; the sheet decides which.
5. **The boots are long forward.** In profile the boot projects well past the shin.

## Keep every one of these from round 4 — they are not in question

- **Box modelling, no lattice.** Rectangular boxes at whatever size the form needs.
- **Every triangle normal exactly ±X/±Y/±Z.** No bevels, smooth normals, rotated boxes, chamfers or
  cylinders. Round 4's exporter **fails the build** on a violation and reported 0 non-axis-aligned
  and 0 smooth-shaded faces. Keep that enforcement.
- **The artifacts of the loop, but NOT the stopping — see "Run it through" below.** One part per
  step, and after every step **save the `.blend`** and write
  `src-assets/renders/progress/NN-<part>.png` showing the WHOLE figure full-size *and* small. Keep
  both; the saves are crash insurance and the renders are the audit trail. What changes this round
  is that you do not wait for a reply.
- **The zoom strip with its speckle metric** — round 4 invented `delta = mean |nearest - mip|` and
  reported 14.1 / 7.3 / 2.1 at 10/30/100 px. Keep the metric; it turns "does it read small" into a
  number.
- **Never touch git.** Write files at the paths named here; the operator commits.
- **Never write outside `src-assets/`.**
- **The naming scheme:** source and export share the stem —
  `src-assets/blender/SM_VoxelDwarf_Miner01.blend`, exported to
  `src-assets/export/SM_VoxelDwarf_Miner01.glb`, revision `r5` in the object, mesh, material and
  image datablock names. Never write to `assets/gltf/`.
- **Parts as separate objects in the `.blend`**, joined into one mesh by `export_dwarf.py`. The
  beard and hair must lift off and leave a complete head underneath.
- **The palette is an input:** the approved ten cells, which the reference sheet's own swatches
  confirm (`Tunic #5F7A6A`). 16 cells is the enforced ceiling, so **6 spare slots** — propose which
  value steps earn them as parts need them. Do not re-derive colour, and do not sample it from the
  video.

## Three corrections to the documents you were given

1. **Check 3 passed.** Round 4 logged skin at 7.4 % against a 10 % target and chose fidelity over
   chasing it — correctly. Re-measured by the orchestrator: the flat pass is antialiased, so exact
   colour matching counts only the pure core of each region. Classified to **nearest palette cell**,
   skin is **13.9 %**. All three checks pass. The check definition in the model sheet is fixed;
   measure shares by nearest cell from now on.
2. **`check_asset.py`'s grid clause will fail a box model, as before** — expected, report it
   verbatim, do not quantise the model and do not edit the checker. Everything else passed on the
   way through last time, and the quad-soup clause passed when probed separately.
3. **The palette ceiling of 16 is an artifact of a bug**, not a design: the clause reads only 4 cells
   of any atlas. Fixing it is owed on the orchestrator's side. Work within 16 for now.

## No budget, no optimisation

Wolf: *"let's keep stretching limits first"*. Round 4 came in at 1,214 triangles against an
eventual LOD0 ceiling of 4,000, so there is no pressure in either direction. **The count is an
outcome and nobody is judging it.** Spend the headroom on form — the stepped crown, the neck, the
shoulder caps, the boot length — and do not optimise anything.

## What comes after this round, so you do not do it now

A rig and a pose, in that order, once Wolf accepts the shape. Round 4 concluded that **stance is the
largest remaining likeness gap** and it is right: every reference frame has him mid-stride with the
lantern raised, and a neutral standing base cannot read like those frames. Joint names are fixed:
`root, hips, spine, chest, neck, head, shoulder.L/R, elbow.L/R, hand.L/R, hip.L/R, knee.L/R,
foot.L/R`, plus `beard`. Rigid weights, one mesh, no quad crossing a joint.

Lighting is Epic 11's and is not yours. The palette revisit is gated on it.

## The endpoint this is building toward

**Hand-author the PARTS, generate the COMBINATIONS.** Wolf wants a unique dwarf per living person,
which is roughly twelve discrete axes plus four continuous dials, with a dwarf stored as a seed.
Round 3 already built the discrete half as `--beard NAME --hair NAME --pose NAME`; round 4's
parts-as-objects `.blend` is the authoring half. Do not design anything in a way that forecloses
either: a part must mount on a declared socket, and a variant must be expressible as a part list
plus a table of hexes.

## Deliverables — the full manifest

| # | what | path |
|---|---|---|
| 0 | **the re-measured proportion table** | in your report, before any geometry |
| 1 | the source, saved after every step | `src-assets/blender/SM_VoxelDwarf_Miner01.blend` |
| 2 | the exporter, one command, enforcing axis-aligned normals | `src-assets/blender/export_dwarf.py` |
| 3 | the exported GLB (gitignored scratch) | `src-assets/export/SM_VoxelDwarf_Miner01.glb` |
| 4 | progress renders, one per accepted step | `src-assets/renders/progress/NN-<part>.png` |
| 5 | the five final views | `src-assets/renders/` |
| 6 | the zoom strip with the speckle metric | `src-assets/renders/zoom-strip.png` |
| 7 | a side-by-side against `dwarf-ortho/side-left.png` | `src-assets/renders/vs-ortho-side.png` |
| 8 | your report | `src-assets/prompts/dwarf-miner-round-5-report.md` |
| 9 | the checker's output, verbatim, pass or fail | in the report |
| 10 | your cost — `session_tokens.py` print mode, verbatim, model from your banner, row `dev-art` | in the report |

Deliverable 7 is new and it is the one that answers Wolf's headline complaint: **the side view is
what was wrong, so the side view is what gets compared.**

## How this round is judged

By Wolf's eye against the sheet, plus these, which are mechanical:

- the crown is stepped, not a single box
- a neck exists
- shoulder caps exist and the torso is wider at the shoulders than at the waist
- the side profile's depths match `side-left.png` proportionally
- the beard is no wider than the head, and as long as the sheet draws it
- visible skin >= 10 % of figure pixels, **measured by nearest palette cell**
- nothing turns to speckle at 10 px on the zoom strip
- every datablock name carries `r5`
- nothing written outside `src-assets/`, and no git command run at all
