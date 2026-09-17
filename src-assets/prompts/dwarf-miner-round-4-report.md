# Round 4 report — the dwarf, box-modelled, live

Session: steered, in Wolf's running Blender through the MCP addon. Eleven modelling steps, each
saved, rendered and shown before the next began. Only the final export ran headless.

## 1. Deliverables

| # | what | path | state |
|---|---|---|---|
| 1 | source, hand-modelled | `src-assets/blender/SM_VoxelDwarf_Miner01.blend` | written, saved after every step |
| 2 | exporter, one command | `src-assets/blender/export_dwarf.py` | written, run |
| 3 | export (gitignored scratch) | `src-assets/export/SM_VoxelDwarf_Miner01.glb` | 86,920 bytes |
| 4 | progress renders | `src-assets/renders/progress/01..11-*.png` | 11 sheets |
| 5 | five final views | `src-assets/renders/dwarf-{flat,lit}-*.png` | 10 files |
| 6 | zoom strip | `src-assets/renders/zoom-strip.png` | written |
| 7 | this report | `src-assets/prompts/dwarf-miner-round-4-report.md` | — |
| 8 | checker output | section 6 below | verbatim, FAIL |
| 9 | cost | section 8 below | verbatim |

The export command, from the repo root:

    blender --background src-assets/blender/SM_VoxelDwarf_Miner01.blend \
            --python src-assets/blender/export_dwarf.py

**Nothing was written outside `src-assets/`. No git command was run at any point.**
`dwarf.blend` was left in place rather than deleted — the new file is modelled from scratch, so the
naming rule is satisfied by the new path, and deleting is the operator's call.

## 2. What the model is

1,214 triangles, against r3's 14,398. Fourteen parts joined at export into one mesh, one material
(`M_VoxelDwarf_r4`), one packed palette image (`T_VoxelDwarf_Palette_r4`). Height 1.2000 m,
min Z 0.000000, centred in X and Y. **Zero non-axis-aligned faces, zero smooth-shaded faces** —
verified on every part after every step and again at export, where it is a build failure.

Parts: `Head Beard Hair Torso Arm.L Arm.R Leg.L Leg.R Boot.L Boot.R Belt Pack Lantern Pickaxe`.
Sockets: `socket.beard socket.hair socket.hand.L socket.hand.R`.

Beard and hair each sit on a declared socket and lift off, leaving a complete head — brow, eyes,
nose, cheekbones, mouth, ears. Progress sheets 03 and 04 render the figure with them hidden to prove
it. Tools are separate objects on hand sockets, never fused.

## 3. Two corrections to the input documents, both raised at the time and both adopted

**The model sheet's headline check was measuring the wrong thing.** Its first edition read
"head + beard mass ends — `ref about 24 %`" against r3's 48.5 %. Measured off the t=10 s frame here:
the **head** ends at 24.7 % (the shoulder line), matching that 24 %, and the **beard tip reaches
42.8 %**, hanging to just above the belt. The 24 % was a head measurement wearing a "head + beard"
label; r3's 48.5 % was a row-majority figure, which measures the beard's WIDTH. Obeying the sheet
would have cut a beard the reference gives to the belt. Sheet and brief were both corrected. This
beard keeps the reference's length and takes in its width.

**"Inherit r3's 23 cells" was not reachable.** Round 4 branches off `main`, where the shipped asset
carries the 10 approved cells and the generator has none of r3's 13 value steps. And the contract as
enforced allows **16 cells**, not 23 (`check_asset.py` fixes 16 px cells in a 64 px atlas). Built on
the 10 approved cells instead, spending spares one at a time as parts needed them.

## 4. The palette — 15 of 16 cells, 1 spare unspent

Every spare bought a **value step**: form painted in rather than borrowed from lighting, which is
what the round asked for. None bought a hue change.

| cell | hex | role | cost |
|---|---|---|---|
| 0–9 | — | the ten approved, meanings unchanged | inherited |
| 10 | `#BAA896` | skin shadow plane | spare |
| 11 | `#826145` | beard highlight | spare |
| 12 | `#7DA18C` | tunic lit plane | spare |
| 13 | `#44584C` | tunic turned-away plane | spare |
| 14 | `#63695B` | trouser lit plane | spare |
| 15 | — | **unspent** | — |

Three values came free by using approved cells cross-purpose: `#34271C` "hair / dark iron" is the
beard's and the boots' shadow and the buckle's inner faces; `#5E4632` beard is the hair's highlight;
`#8B6B50` / `#6B5B49` carry all the leather. Boots, belt, pack, lantern and pickaxe cost **zero**
slots between them.

Exported palette, in atlas order:
`#E9D2BB,#5E4632,#FFFFFF,#5F7A6A,#474B41,#A9B2AC,#8B6B50,#6B5B49,#34271C,#F0A63C,#BAA896,#826145,#7DA18C,#44584C,#63695B`

## 5. The three numeric checks, measured on `dwarf-flat-front.png`

Measured on the **flat albedo** pass, because the lit pass shades colours off their exact palette
values — the asymmetry the sheet describes. The band check uses the sheet's own window: the body
centre band, **excluding held tools**, because it is a chest-coverage measure and a held pickaxe
must not be allowed to decide it.

| check | target | r3 | r4 |
|---|---|---|---|
| 1 — beard no wider than the head | — | crossed the chest | **0.25 m vs 0.336 m — PASS** |
| 2 — tunic wins at least 4 body-band tenths | >= 4 | 1 | **4 — PASS** |
| 3 — visible skin | >= 10 % | 6.8 % | **7.4 % — MISS** |

Body-band tenths (brown / tunic): 0–10 45.9/0, 10–20 14.8/0, 20–30 60.6/0, 30–40 62.3/5.2,
**40–50 8.9/55.0**, **50–60 16.0/22.4**, **60–70 4.6/42.6**, **70–80 0.0/4.0**, 80–90 60.9/0,
90–100 63.2/0. Brown owns the head and beard down to 40 %, tunic takes over from there — the beard
tip at 42.8 % landing where the reference puts it. Whole-figure shares: brown 29.1 %, tunic 18.7 %,
grey 8.1 %, skin 7.4 %, flame 1.0 % — against r3's brown 59.2 % / tunic 22.5 %.

**Check 3 misses, and I stopped pushing deliberately.** It went 3.3 % to 8.4 % across the arms step
by three reference-measured changes: the bare forearm extended to the contact sheet's 67 % of height;
the forearm made THICKER than the sleeve (I had given it human proportion, which is wrong for a dwarf
and threw away skin); and the beard's cheek mass narrowed at the moustache line, where it had been
burying cheeks the reference leaves bare. It then fell to 7.4 % when the pickaxe and lantern joined
the denominator. Past that point the only remaining lever is **less hair and beard coverage than the
reference has** — the face opening is already at the reference's 63 % of head width and the arm is at
reference proportion. Recorded as a miss against a target set by estimate: the sheet is explicit that
the reference's own shares are not recoverable from the video.

## 6. `check_asset.py` — verbatim, and it FAILS by design

    $ python scripts/bench/check_asset.py src-assets/export/SM_VoxelDwarf_Miner01.glb
    FAIL src-assets\export\SM_VoxelDwarf_Miner01.glb: grid clause: POSITION values must use the 0.0125 m project grid
    EXIT=1

This is the failure the brief predicted: the grid clause asserts every position sits on the authored
lattice, and a box model has no lattice. **Not quantised to satisfy it; the checker was not edited.**
Worst off-lattice offset is 6.000 mm against a 0.010 mm tolerance. The replacement guarantee on this
side is the axis-aligned-normal rule, which the exporter enforces and fails the build on: **0
non-axis-aligned faces, 0 smooth-shaded**.

The checker stops at the first failure, so the clauses *after* the grid clause were probed separately
using the checker's own module, unedited:

- **quad-soup clause — PASS**: 1,214 tris, 2,428 verts, required `tris/2*4` = 2,428. No vertex is
  shared between quads, so no normal is ever averaged across a hard edge.

Everything before the grid clause passed on the way through: file, one-primitive, single-sided,
palette/material (64x64 atlas, no holes, NEAREST filtering, CLAMP_TO_EDGE, UVs inside 0–1, sole
material), naming (mesh and node agree: `SM_VoxelDwarf_Miner01_r4`), applied identity transform,
finite positions, and no-extensions.

One real bug found and fixed on the way: the first export carried `KHR_materials_specular` and was
rejected by the no-extensions clause. Cause was a Specular IOR Level of 0 set on the material at step
2; returning it to the 0.5 default removes the extension and changes nothing for a flat-shaded asset.
Backface culling was also enabled, so `doubleSided` is false as the single-sided clause wants.

## 7. Where the model departs from the reference, and why

- **The bedroll runs along X, not diagonally.** The reference slings it across the back at an angle;
  a diagonal box is a rotated box, which the invariants forbid outright. Its rolled ends face +/-X,
  where two concentric steps read as the coil. Same reason the chest straps are vertical bands.
- **There is no shoulder to route a strap over.** A 0.336 m head against a 0.350 m torso leaves 7 mm
  outboard — the chunky proportion the reference has leaves literally nowhere for an over-shoulder
  strap to pass. Straps terminate at the shoulder line.
- **The pick head runs fore-aft, not broadside.** Broadside reads better from the front, but on a
  hanging carry the inner arm passes straight through the torso. Fore-aft is the only collision-free
  axis-aligned option; it reads in side and three-quarter views.
- **The pickaxe arc is a staircase of four boxes.** The reference's is a smooth curve, which a box
  model cannot have. This is the idiom, not a compromise of it.
- **The buckle is grey iron, not brass.** The reference's buckle reads warm. Correcting that is a HUE
  change, not a value step, so the last spare slot was left unspent rather than used to re-derive a
  colour the brief says not to re-derive. `#F7CE94` from the sheet's 13 is the obvious candidate if
  Wolf wants it.

## 8. Cost — `session_tokens.py`, print mode, verbatim

Run without `--story`, which is what enables ledger recording, so nothing was written outside
`src-assets/`. Row label `dev-art`. Model read from the session banner: **claude-opus-5**.

    Session token cost  (db3e163e-83c1-4093-90fb-0a33a2ab15e2.jsonl, tool=claude)  (441 turns, claude-opus-5)
      input (fresh)            882
      cache creation       650,094
      cache read        99,577,271
      output               604,895
      total processed  100,833,142
      wall-clock           119 min  (elapsed, includes idle gaps)
      est. cost             $68.98  (benchmark — verify rates in PRICES)
      (pass BOTH --story and --phase to record a ledger row)

`session_tokens.py --tool claude` could not find the transcript on its own on this Windows clone: it
builds the project directory as `C:\Users\suihk/.claude/projects/D:\Workspace\frostvein` instead of
the escaped slug `D--Workspace-frostvein`. Passed `--transcript` explicitly. Owed work on the
orchestrator's side, not fixed here.

## 9. What limited the likeness — the things that fought the reference

Ordered by how much each actually cost the result, not by how hard each was to work around.
Sections 3 and 7 above record individual departures; this is the general case.

**1. Lighting is where half the reference's appeal lives, and it is out of scope.** Every reference
frame is torch-lit: warm bounce, soft shadows, a lantern pooling light on the floor, strong value
falloff across a single surface. Our renders are Workbench flat/studio on grey `#6F7073`. The round
therefore compared an unlit asset against a lit hero shot — structurally different things. The brief
is right that lighting is not the art seat's, but the consequence is that a large part of "does it
feel like the reference" is not addressable from inside this round at all.

**2. Flat per-face colour cannot do gradation, so form must be quantised into planes.** The
reference's tunic shades continuously across one surface. A box model cannot: colour is one cell per
face. The only way to add form is to subdivide a face into colour regions, and that tips into reading
as STRIPES very easily. Two builds were thrown away to this: the beard's first version (horizontal
bands, every band's top an exposed lit ledge) and the hair's first version (one value, so the stepped
fringe was present and invisible). The line between "value step" and "corduroy" is narrow and there
is no way to soften it.

**3. The 16-cell palette ceiling.** Fifteen used. The tunic alone needed three greens, skin two,
beard three. Boots, belt, pack, lantern and pickaxe cost zero slots ONLY by reusing approved cells
cross-purpose — `#34271C` "hair / dark iron" serves as beard shadow, boot shadow and buckle interior
simultaneously. What could not be afforded: a brass buckle, a third leather value, eye detail, any
warm/cool variation. The reference carries far more gradation than fifteen flat cells can.

**4. "No rotated boxes" blocked specific reference features outright.** Not a style to work within —
a wall hit four times: the diagonally slung bedroll, the diagonal chest straps, the pickaxe head's
smooth arc, and the figure's stance. The reference has subtle tilt and asymmetry that reads as alive;
everything axis-aligned reads stiff and mirror-symmetric. Partly fought with asymmetric lock lengths
and deliberately non-mirrored hair, but only partly. Related and worth stating plainly: **the
reference is not itself a strict box model** — it has real curves and soft normals — so "match it
with axis-aligned boxes" has a built-in ceiling.

**5. The reference is always POSED; this is a neutral base.** Every frame has him holding the lantern
up, pickaxe shouldered, mid-stride. Much of the silhouette's appeal is the pose. A neutral standing
asset with arms down will not read like those frames, and rigging is last so posing was not available.
The comparison is unfair to the base asset in a way that will not resolve until there is a rig.

**6. The reference is a poor measuring surface, and it produced one wrong document.**
Only one usable frame (t=10 s), 3/4-turned and in perspective, so every width is foreshortened and
**every depth (Y) dimension in this model is an estimate** — there is no orthographic sheet. The
contact sheet is 320x180 per cell, fine for gear inventory and useless for proportion. There is **no
back view at any usable resolution**, so the back of this dwarf is substantially invented. And under
torch light the green tunic reads olive-brown, **hue-identical to leather**: an attempt to reproduce
the sheet's band measurement on the reference itself failed because no classifier can separate tunic
from belt in that footage. That is the same weakness that produced the sheet's incorrect headline
check (section 3).

**7. One numeric check pulled against fidelity.** Check 3 (skin >= 10 %) is a target set by estimate,
not measured off the reference. It went 3.3 % to 8.4 % on genuine fixes, then to 7.4 % when the tools
entered the denominator. The only remaining lever was giving him LESS hair and beard than the
reference has. Fidelity was chosen and the miss logged, but this is an unresolved conflict between
two of the round's own inputs, not a solved problem.

**8. Proportion forecloses gear the reference has.** A 0.336 m head against a 0.350 m torso leaves
7 mm of shoulder — nowhere to route an over-shoulder strap, which the reference clearly has. Either
its head is narrower than measured here or its straps cheat. Not reconciled; worked around.

**9. The authoring interface cannot model by eye, and that compromised the round's premise.** Every
change through `execute_blender_code` is compute-coordinates, run, render, look, adjust. No dragging
a vertex. That made each aesthetic iteration expensive and capped how many were affordable per part.
More importantly: because hand-editing is impractical through this interface, the parts ended up
built by `build_head()`, `build_beard()` and so on in the `.blend`'s text datablocks — **which is a
generator again**, only smaller and more legible than `dwarf_miner.py` was. The "hand-modelled"
premise is therefore only partly realised. The way to get genuinely hand-tuned geometry is for Wolf
to nudge vertices directly and for the seat to be told which parts to stop regenerating.

**What did NOT limit anything**, recorded for balance: there was no triangle pressure (1,214, well
inside even the LOD0 ceiling of 4,000), no file-size pressure (86,920 bytes against a 16 MB limit),
and the grid-clause failure was expected and harmless.

## 10. Recommended next, in order of expected gain

1. **A rig and a pose.** The single biggest remaining likeness gap is stance, not geometry
   (section 9.5). Joint names are already fixed in the brief. Expect this to close more of the
   perceived distance to the reference than any further modelling would.
2. **Epic 11's lighting, before any further palette judgement.** Half the reference's feel is its
   torch-lit scene (section 9.1), and the palette revisit is already gated on this. Re-judging albedo
   under the finished renderer is the point at which the remaining colour questions — the brass
   buckle, the warmer tunic — become answerable instead of guesses.
3. **An orthographic reference turnaround: front, side, back.** This removes most of section 9.6 at a
   stroke — every depth dimension in this model is currently estimated and the back is invented. It is
   the cheapest single input that would raise fidelity across every future part and every variant.

Beyond those three: fix `check_asset.py`'s palette clause so the atlas ceiling reflects reality, which
is what would let the tunic and leathers carry the gradation section 9.3 could not afford.

## 11. Owed work and notes for whoever picks this up

- **`render_dwarf.py` still imports the retired `dwarf_miner`**, so running it would render the OLD
  dwarf. The five final views here were written from the `.blend` at the same paths and names that
  script uses. It needs repointing at the `.blend`; not done, as its ownership sits outside this round.
- **The model sheet's feature table understates the belt.** Measured at 6x zoom: strap 39 px =
  **4.9 units**, buckle 47 px = **5.9 units**. The sheet says 2 and 5. Its buckle figure is right;
  its belt figure is out by about 2.5x. The numbers are in `dwarf_parts.py`'s comments.
- **`check_asset.py`'s palette clause reads only 4 cells** — which is why r3's 23-cell atlas passed.
  Already filed on the orchestrator's side; noted again because the 16-cell ceiling is real until it
  is fixed, and this asset now uses 15 of them.
- **No rig.** The brief makes it optional and last; the shape was still being corrected through step
  11, so rigging was not started. Joint names are fixed in the brief for whenever it happens.
- **No decimation, no LODs, nothing optimised** — explicitly out of scope. At 1,214 triangles the
  asset is already inside the LOD0 ceiling of 4,000, so the decimation task the model sheet
  anticipated may not be needed at LOD0 at all.
- **Facing.** He faces +Y in Blender, matching the shipped asset and `render_dwarf.py`. That maps to
  **-Z in glTF**, which is backwards from the glTF convention of characters facing +Z. Kept for
  consistency; worth a decision before animation.
- **Authoring helpers live in the `.blend`** as text datablocks `dwarf_build.py`, `dwarf_parts.py`,
  `dwarf_render.py`. They exist so a part can be re-derived after a correction without hand-editing.
  They are NOT a generator — the `.blend` is the source of truth.
