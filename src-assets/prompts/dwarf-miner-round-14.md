# Round 14 — from scratch, from the references, as parts, judged by silhouette

**This round replaces the method.** Rounds 9–13 carved one continuous body, one operator per call,
and were scored on a plane count. Round 13 executed that spec well and still does not look like the
reference, because the gap is proportion, silhouette and paint, none of which a plane count
measures. The plane metric is retired. Carving is retired. The generator ban is lifted.

**You start from nothing.** Do not open, read, copy from or reference any previous `.blend`, GLB,
generator, prompt or report. The only inherited document is `references/dwarf-model-sheet.md`,
because its orthographic table *is* the reference as numbers. Everything else comes from the
reference directory.

---

## 1. What the reference is

Look at it before reading any number. `references/` holds more than the two images earlier rounds
leaned on, and you are expected to use all of it:

| input | what it is | use it for |
|---|---|---|
| `references/dwarf-ortho/` + README | front, side-left, side-right, back, 5× nearest-neighbour crops of the sheet | **the geometry authority**: heights, widths, depths, silhouette |
| `references/reference-sheet.jpg` | the orthographic model sheet the video was rendered from, with gear breakdown and palette | palette hexes, what gear exists; **ignore its printed dimension labels** |
| `references/dwarf-frames/` (`f084`–`f164`) + README | 240 frames of the lit turntable | **form, construction, lighting**: `f088` depth, `f104` face, `f084` domed stepped crown, `f140`/`f164` neck and shoulder caps. Open at least those five and skim the rest |
| `references/dwarf.mp4` | the source video | the look; **never measure from it** |
| `references/dwarf-contact-sheet.jpg` | palette and gear authority | the approved ten hexes |
| `references/dwarf-model-sheet.md` | the measured tables | the numbers, §3 |

What the references show, in words: a figure built from about a dozen chunky parts. A head that is
very nearly a cube and roughly a third of the figure's height, with a nearly flat face, a stepped
domed crown and a brow ridge. A beard **as wide as the head and no wider**, hanging to just above
the belt, with the tunic, belt and buckle visible either side of it. A hair cap with a stepped
silhouette that parts around the neck so the neck is bare in profile. Eyes, brows and mouth that are
painted. Shoulders carried by the sleeve caps, well outboard of the skull. Torso, skirt, sleeves,
gloves, legs and boots as boxes with two or three steps each. A small pack behind the shoulders.

The reference has a few hundred distinct surface directions in total. **More geometry is not the
fix. Correct masses in correct places are.**

## 2. The one gate: the silhouette matches the ortho sheet

Render the figure orthographically from the same four cameras as `dwarf-ortho/`, overlay it on the
sheet at matched scale, and judge per part. Head, hair, beard, torso, skirt, boots and pack land on
the sheet's outline within about one source pixel (8.571 mm). The arms are exempt (§4, pose).

Three checks from the model sheet are **reported**, because they are silhouette-derived and track
the reference; they are not tuned to:

- **neck bare in profile**: a run of skin ≥ 0.08 H tall between hair and collar in both side views
- **silhouette step density** ≥ 10 steps per 100 rows on front and side, neither edge below 70 %
  of the other
- **visible skin** ≥ 10 % of figure pixels, classified to nearest palette cell

No plane count. No triangle target. No median edge length. The exporter still prints those lines;
**do not act on them.** If its form floor (1,200 planes) fails the build, set that one constant in
`src-assets/blender/export_dwarf.py` to 0 and say so. That is the only tooling edit this round.

---

## 3. The numbers — from `dwarf-model-sheet.md`, orthographic table, settled

**Do not re-measure.** 140 source pixels = 1.00 H = 1.200 m; one pixel is 8.571 mm. `front.png`
column 77 is the centre line, `side-left.png` column 58 the depth centre, +Y forward, the figure's
right is +X, sole at z = 0.

**Heights, z / H from the sole:**

| landmark | z / H |
|---|---|
| crown | 1.000 |
| crown step 2 / step 1 / main skull top | 0.979 / 0.964 / 0.943 |
| brow | 0.879 |
| ear top / bottom | 0.850 / 0.729 |
| eye line | 0.807 |
| nose tip, lowest point | 0.750 |
| head + hair mass ends | 0.707 |
| shoulder line, top of sleeve cap | 0.700 |
| neck visible in profile | 0.793 → 0.679, up to 0.064 H wide |
| sleeve cuff, forearm begins | 0.566 |
| beard tip | 0.464 |
| belt top / bottom | 0.421 / 0.343 |
| pack bottom | ~0.35 |
| hand bottom | 0.330 |
| tunic hem | 0.207 |
| boot cuff top / bottom | 0.164 / 0.107 |
| sole | 0.000 |

**Widths, / H:** head with hair 0.336; ear to ear 0.383; crown steps top-down 0.164 / 0.229 /
0.286 / 0.336; **beard widest 0.343** (the head's own width); chest, tunic only 0.317;
**shoulders over the sleeve caps 0.528**; waist / skirt 0.439; stance boot-outer to boot-outer
0.398; one leg or boot 0.164; boot cuff 0.200.

**Depths, / H, +Y forward:** head back-to-hair-front 0.336 (nearly a cube); nose tip past the back
of the head 0.379; beard front 0.357; torso 0.286; skirt 0.300; pack behind the torso back 0.186;
shin 0.164; boot sole length 0.214, toe projecting 0.057 past the shin front; whole figure pack to
nose 0.550. The hair's front edge in profile is `y = +0.111`.

**Feature scale:** 1/64 H (~19 mm) is the reference's finest authored detail. Belt buckle ≈ 5 units
across, belt band ≈ 2 units tall. There is no lattice; use this to size gear, not to snap positions.

**Two consequences that earlier rounds got wrong and this table settles:** the shoulders are
0.096 H outboard of the skull on each side, carried by the sleeve caps, not the torso; and the
beard is long but **narrow** — it never crosses the chest.

---

## 4. Method

**One generator script, `src-assets/blender/dwarf_r14.py`.** A `PARAMS` dict (every dimension in
metres derived from §3, every colour as hex) and a `build()` that deletes collection
`SM_VoxelDwarf_Miner01_r14` if present and rebuilds it. Iteration means: change a parameter, run
the script in the live Blender, render the overlay, look. Each run is one tool call.

**Start from an empty scene** in the live Blender (`wm.read_homefile`, delete the default cube),
save to `src-assets/blender/SM_VoxelDwarf_Miner01.blend`. Round 13 is in git; the operator owns it.

**Parts are separate objects.** Stacking primitives is the method, because that is how the
reference is built. The parts:

```
r14_head   r14_hair   r14_beard   r14_moustache
r14_torso  r14_skirt  r14_belt    r14_buckle
r14_sleeve.L/R  r14_glove.L/R
r14_leg.L/R     r14_boot.L/R
r14_pack   r14_strap.L/R   r14_lantern  r14_pickaxe
```

Each part starts as a cube scaled to §3, then gets at most a handful of operations: `inset`,
`extrude`, `scale` of a selected loop, `bevel` on a chosen edge set at 10–25 mm. A part that needs
more than about eight operations is being over-built.

**Per-part recipe, judged against `f088` / `f104` / `f084`:**

- **head**: 0.336 × 0.336 cube-ish, main skull top at 0.943. Front face flat. One brow band
  extruded ~15 mm forward across the full width at 0.879. One nose block ~40–50 mm proud, tapering
  to a tip at 0.750–0.79. Ears as small tabs ≤ 15 mm proud, 0.729–0.850. **No eye sockets, no
  cheekbones, no jaw carving.**
- **hair**: crown steps at 0.943 / 0.964 / 0.979 / 1.000 with widths 0.336 / 0.286 / 0.229 /
  0.164; a cap over the top and back, side lobes ending at `y = +0.111`, **parted either side of
  the neck** so 0.08 H of neck shows in profile. It does not cover the face.
- **beard**: 0.343 wide at the top, tapering, tip at 0.464, front at `y = 0.357 − 0.336/2` from the
  head's back plane, 3–5 steps. Tunic and belt must show either side of it.
- **moustache**: one flat wedge over the mouth line.
- **torso / skirt**: boxes, chest 0.317 wide and 0.286 deep, skirt flaring to 0.439 at the hem
  (0.207). Belt box at 0.343–0.421, buckle box ≈ 5/64 H across.
- **sleeves**: caps top out at 0.700 and carry the 0.528 shoulder width; cuff at 0.566. Bare
  forearm below in skin.
- **gloves**: box, bottom at 0.330, one thumb block.
- **legs / boots**: 0.164 wide each at 0.398 stance; boot has sole 0.214 long with 0.057 toe
  projection, cuff 0.200 wide at 0.107–0.164.
- **pack**: one box 0.186 deep behind the torso, no wider than the chest, bottom ~0.35. Straps as
  thin boxes on the sleeve caps, never outboard of the neck.
- **lantern, pickaxe**: simple; pickaxe shaft 0.91 H, blade span 0.38 H, head-span/length 0.42,
  from `front.png`. Held in neutral hands: pickaxe right, lantern left, in contact.

**Pose:** A-pose, arms out ~40°, ≥ 20 mm armpit clearance. The sheet is posed; its arm span is a
pose, not the body. Silhouette comparisons will not match at the arms. Expected.

**Rules kept from earlier rounds:**
- author inside Wolf's running Blender through the MCP addon; prove the live scene first
- drive `areas[0]` only, name the run in the status bar, screenshot after every run, save after
  every run
- nothing written outside `src-assets/`; no git commands of any kind, read-only included
- no culling of buried faces; parts within one object overlap, never meet exactly

---

## 5. Stages and two checkpoints

**Stage A — blockout.** Script, all parts, grey material, overlay renders on all four ortho
views, `f088` / `f104` side-by-sides. **Stop here and show Wolf.** Do not texture, do not rig, do
not fix anything Wolf has not seen. Expect one or two rounds of parameter changes here.

**Stage B — paint.** One material `M_VoxelDwarf_r14`, one image `T_VoxelDwarf_r14`, 512 × 512,
packed. **UVs laid out by the script, not by `smart_project`**: the face front gets a fixed
rectangular island so the eyes cannot drift between runs. Base colours are the approved ten
(`#E9D2BB` skin, `#5E4632`, `#FFFFFF`, `#5F7A6A` tunic, `#474B41`, `#A9B2AC` metal, `#8B6B50`
wood, `#6B5B49`, `#34271C` hair, `#F0A63C` flame) with crisp value steps by face orientation, no
gradients. Paint the eyes big and dark with a white highlight, heavy brows, a mouth line, cheek
warmth, the buckle, boot dirt. Judge the face against `f104` at 100 px and 60 px. `Closest`
interpolation, backface culling on, Specular IOR Level 0.5. **Stop and show Wolf.**

**Stage C — rig and export.** `SK_VoxelDwarf_Miner01_r14` to the unchanged 19-joint contract
(`root hips spine chest neck head shoulder/elbow/hand.L/R hip/knee/foot.L/R beard`). Rigid
weights, one joint per vertex at 1.0, assigned per part by the script. Join parts at export. One
deflection render per joint group, the two-handed carry as a posed render, GLB in bind pose, no
animation. `check_asset.py` exit 0.

---

## 6. Budget

- **≤ 150 tool calls for the whole round.** If Stage A is not converging by call 60, stop and
  report what is not landing.
- No bevel added to reach a number. No operation added because a metric moved.
- **Fewer triangles is never a failure.** Expect 1,500–4,000.

## 7. Deliverables

| # | what | path |
|---|---|---|
| 0 | live-scene report before the first run | in the report |
| 1 | the generator script | `src-assets/blender/dwarf_r14.py` |
| 2 | the `.blend`, saved per run, image packed | `src-assets/blender/SM_VoxelDwarf_Miner01.blend` |
| 3 | overlay renders, four views, each checkpoint | `src-assets/renders/r14/overlay-<A|B>-{front,side-left,side-right,back}.png` |
| 4 | beside `f088` and `f104`, matched scale | `src-assets/renders/r14/vs-frames-*.png` |
| 5 | face close-up, readability at 100 px and 60 px | `src-assets/renders/r14/face-*.png`, `readability.png` |
| 6 | deflection and carry renders | `src-assets/renders/r14/joint-*.png`, `pose-*.png` |
| 7 | texture | `src-assets/blender/textures/T_VoxelDwarf_r14.png` |
| 8 | report, **under 150 lines**: `PARAMS`, which silhouettes land and which do not, the three §2 checks, exporter and checker output verbatim, cost row `dev-art` | `src-assets/prompts/dwarf-miner-round-14-report.md` |

## 8. How this round is judged

By Wolf's eye against the frames and the overlays. Then: silhouette within one source pixel on
head, hair, beard, torso, skirt, boots, pack in all four views; neck bare in profile; face readable
at 60 px; zero exposed holes, no inside-out mass, rig contract met, `check_asset.py` exit 0; under
150 calls with both checkpoints honoured.

## 9. Not in this round

Animation, LODs, the plane-of-form metric in any form, and anything from a previous round's
geometry.
