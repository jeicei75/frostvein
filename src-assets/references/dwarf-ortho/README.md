# The orthographic views — cropped from `reference-sheet.jpg`, which is the UPSTREAM source

**Read this before measuring anything.** `src-assets/references/reference-sheet.jpg` is the
**modelling reference sheet the video was rendered from.** It carries orthographic **front**, **both
sides**, and **back** views, a gear breakdown with proportions, and the palette with hex codes.

Everything measured for rounds 1–4 was measured from `dwarf.mp4` instead — a lit, perspective,
h264-compressed render **generated from this sheet**. Two generations downstream of an orthographic
source that was sitting in the same directory. That is why round 4 had to estimate every depth
dimension and invent the back, and why the model sheet's proportions carry a ±4 % caveat they never
needed to.

These crops are 5x nearest-neighbour blow-ups of the sheet's own pixels — no resampling, no
interpolation, so every original pixel is a clean 5x5 block you can count.

| file | what it is authoritative for |
|---|---|
| `front.png` | front proportions, the face, shoulder width against head width |
| `side-left.png` | **every depth (Y) dimension** — head and beard projection, torso depth, the pack, boot length forward. Round 4 estimated all of it |
| `side-right.png` | cross-check for the left side, and the lantern arm |
| `back.png` | the back, the pack's straps and flap — **previously invented outright** |
| `gear.png` | the pickaxe and lantern, with the sheet's own proportions: **pickaxe 0.83x dwarf height, lantern 0.33x** |

## What the sheet settles that four rounds of video measurement could not

- **The head is not a box.** The crown is **stepped and domed**, visible in both side views — it
  steps in at the front and the back. Within "no rotated boxes" this is reachable by stepping the
  crown with two or three smaller boxes.
- **There is a neck**, and **there are shoulder caps** — the sleeve sits on the shoulder and the
  torso is wider at the top than at the waist.
- **The body has real depth.** The side silhouette is a deep torso with a forward-projecting nose,
  brow and beard, not a flat slab.
- **The boots are long forward.** In profile the boot projects well past the shin — which is the
  "feet could be a bit longer" note, and it is fidelity rather than a departure.
- **The palette on the sheet IS our approved ten** — `Skin #E9D2BB`, `Beard #5E4632`,
  `Snow #FFFFFF`, `Tunic #5F7A6A`, `Pants #474B41`, `Metal #A9B2AC`, `Wood #8B6B50`,
  `Wood Trunk #6B5B49`. So the approved cells came from here, and **"the reference looks warmer than
  our palette" was an artifact of the one dim video frame it was asked about.** That question is
  closed until Epic 11, as ruled.

## One warning about the sheet

**Measure the ART, not the annotations.** At least one label is self-contradictory: the vertical
dimension on the front view reads as `0.6x dwarf height` for the dwarf's own height. Others may be
decorative too — `12 Voxels` across the body would imply a ~15-voxel-tall dwarf, which the sheet's
own artwork plainly exceeds. The drawings are orthographic and trustworthy; the numbers written
beside them are not.

And it is still a **1024x558 JPEG**, so each orthographic view is only ~110–160 px tall in the
original. Proportions as fractions are solid; single-pixel features are not.
