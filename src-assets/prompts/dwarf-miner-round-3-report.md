# Round 3 report — the `dwarf.mp4` look, a skeleton, and a scheme for hundreds

Answering `dwarf-miner-round-3.md`. Everything below was produced on the art seat under
**Blender 5.2.1 LTS** (build 2026-08-25, hash `9e2066aef7ef`), Windows 11.

**Nothing outside `src-assets/` was written.** `scripts/bench/check_asset.py` was run, not
edited — and it has more to say than the brief expected. See §5.

---

## 1. What shipped

| file | sha256 (first 16) | bytes |
|---|---|---|
| `src-assets/candidates/SM_VoxelDwarf_Miner01_r3.glb` | `dd34c1b3154a4cba` | 1,399,040 |
| `src-assets/candidates/SM_VoxelDwarf_Miner01_r3_swing.glb` | `1c3a5a35242b41ea` | 1,400,936 |

Both regenerate **byte-identical on a cold Blender**, verified by two fresh runs with
`__pycache__` removed between them; the committed files' hashes are the run-1 and run-2
hashes. There is still no `--seed`: the generator draws no random numbers at all.

```
FIGURES name=SM_VoxelDwarf_Miner01_r3 revision=r3 pose=neutral beard=miner-full hair=miner-shag
voxel=0.0125 height_voxels=96 voxels=150340
groups=arm:11711,beard:15794,belt:10146,hair:7368,head:22990,lantern:7584,leg:17911,pack:24625,pickaxe:3800,torso:28411
joints=19 centring=-5,+9 quads=5529 verts=22116 tris=11058 bbox=1.2500x1.2000x0.8000
centre_x=+0.000000 centre_z=+0.000000 min_y=+0.000000 volume=0.293633 expected_volume=0.293633
materials=1 primitives=1 images=1 cells=23
palette=#E9D2BB,#BAA896,#5E4632,#826145,#423123,#FFFFFF,#5F7A6A,#7DA18C,#44584C,#474B41,#63695B,#A9B2AC,#CBD6CE,#707572,#8B6B50,#AF8765,#6B5B49,#8F7A62,#493E32,#34271C,#513C2B,#F0A63C,#F7CE94
glb_bytes=1399040
OK SM_VoxelDwarf_Miner01_r3 (r3, neutral) -> ...\candidates\SM_VoxelDwarf_Miner01_r3.glb
```

The swing GLB carries the identical figures except `pose=swing` and `glb_bytes=1400936` —
**identical geometry, because a pose is node transforms and not a second mesh.** Its
`POSITION` accessor is the rest mesh, so it passes the grid, centring and volume clauses
exactly as the neutral one does.

Renders, all in `src-assets/renders/`: five views x (flat, lit) neutral, five lit in the
swing pose, `dwarf-zoom-strip.png`, `dwarf-vs-dwarf-mp4.png`, and the inherited
`dwarf-vs-contact-sheet.png`. `FLAT-CHECK all 23 palette colours reach the PNG exactly` —
run under **Workbench**, which is what round 2 left owed from this seat and which now
prints clean with the new cell count in place of the 10.

---

## 2. The seven asks

**1. The groove passes are gone.** `groove()` is deleted, not disabled, and the claim is
checked on every build rather than asserted: `find_repeating_channel()` takes the columns
whose outermost voxel sits behind both of their neighbours' in at least six z rows, and fails
the build if four or more of them form an unbroken run at one stride between 2 and 8 while
owning the span they cover. Both sides of the figure are scanned, because r2 cut the pack's
channel from behind.

**That check took three versions, and the first two called a deliberately grooved body
clean.** It is worth recording, because a check nobody has seen fail is not evidence:

- grouping notched columns by `(colour, z)` cannot see **r3's own beard** grooved. The beard
  carries three cells now, so a channel cut through its lit front leaves no lit voxel in that
  column rather than a lowered one, and a per-colour row simply has a gap where the notch is;
- scoring "what share of all notched columns sit at this stride" cannot see it either. The
  figure has honest notches everywhere — the crease at each arm, the gap between the boots —
  so a groove over one mass is a minority of the body's notches however regular it is;
- a share over the whole spread of a stride's hits **missed the pack groove by two
  hundredths**: eleven channels at a stride of 3, two of them isolated and two honest notches
  falling inside their spread, giving 0.79 against a 0.8 bar.

Verified by sabotage in both directions. Reinstating r2's exact pass:

```
CLEAN   find_repeating_channel -> None
GROOVED beard, front, stride 3 -> (1, 3)
GROOVED beard, front, stride 5 -> (1, 5)
GROOVED pack,  back,  stride 3 -> (-1, 3)
```

**2. Twenty-three cells, and the form is painted with them.** The ten ruled hexes are
carried verbatim. The thirteen new ones are **value steps computed from them by a stated
factor**, not measurements — nothing survives being sampled off a 3.4 MB h264 encode of a
torch-lit scene, and the brief says so itself. `step()` scales a hex, which keeps its hue
exactly (the method r2's HAIR cell was derived by) and **refuses to clamp**, so a factor that
would blow a channel past 255 raises instead of silently desaturating. One cell uses
`tint()` toward white instead, and says why.

| # | hex | role | # | hex | role |
|---|---|---|---|---|---|
| 0 | `#E9D2BB` | Skin | 12 | `#CBD6CE` | Metal, highlight |
| 1 | `#BAA896` | Skin, in shadow | 13 | `#707572` | Metal, underside |
| 2 | `#5E4632` | Beard, mid | 14 | `#8B6B50` | Wood |
| 3 | `#826145` | Beard, lit lock | 15 | `#AF8765` | Wood, lit face |
| 4 | `#423123` | Beard, outline lock | 16 | `#6B5B49` | Leather, mid |
| 5 | `#FFFFFF` | Snow / eye white | 17 | `#8F7A62` | Leather, cuff and top edge |
| 6 | `#5F7A6A` | Tunic, mid field | 18 | `#493E32` | Leather, sole and shadow |
| 7 | `#7DA18C` | Tunic, hem and lit band | 19 | `#34271C` | Hair / dark iron |
| 8 | `#44584C` | Tunic, fold and sleeve | 20 | `#513C2B` | Hair, crown |
| 9 | `#474B41` | Trouser / lantern iron | 21 | `#F0A63C` | Lantern flame |
| 10 | `#63695B` | Trouser, lit | 22 | `#F7CE94` | Lantern flame, hot core |
| 11 | `#A9B2AC` | Metal | | | |

Assignment is by **form**, never by noise: the tunic's three greens are a hem band that
follows the hem's own step, a mid field, and a dark fold down each side and across the
shoulders and sleeves; the beard is dark at the outline and at each lock's tip, mid in the
mass, lit on the upper front lobes; leather is dark at the sole, mid in the shaft, lit at the
cuff and along the belt's top edge. Every one is a large area, and every one survives the
downscale to 30 px in the zoom strip.

**One cost to name.** The atlas is still the contracted 64x64 image, but it is now an **8x8
grid of 8 px cells** rather than 4x4 of 16 px, because a 4x4 grid holds sixteen and this
palette needs twenty-three. That is the direct cause of the checker finding in §5.

**3. Detail is in the silhouette.** Four tables, all functions of `|x|` alone and all coarse
on purpose — `BEARD_LOCKS`, `HAIR_CROWN`, `HAIR_LINE`, `TUNIC_HEM`, plus `BOOT_CUFF_TOP`.
They cut the outline rather than the surface: the beard's bottom edge is five locks of
different lengths, the crown steps, the hairline steps across the temples, the tunic hem
steps, the boot cuffs step. The reasoning is the brief's and is repeated in the source: a
3-voxel step in an outline survives a nearest-neighbour downscale to 10 px as a readable
notch; a 1-voxel surface channel does not survive it at all, it aliases.

The buckle is now a **shape** — a 6x4 plate standing two voxels proud of the belt band with a
dark tongue slot — so it notches the profile instead of being a painted face.

**4. Held at 96, and the reason is mechanical rather than aesthetic.** The brief leaves the
call here and says the count may not go up. It cannot usefully come down either:
`check_asset.py`'s grid clause requires every `POSITION` to be a multiple of
`PROJECT_GRID_METRES = 0.0125`, the voxel is `1.20 / N`, so **N must divide 96**. The
reference's ~60-65 is unreachable — 60, 64 and 65 all put every vertex off the project grid
— and the only other candidate, 48, loses the eyes, the brow ledge and the nose, which is
exactly the loss v1 reported at 12 voxels. So the choice was 96 or a contract change, and a
contract change is not this round's to make. This is recorded as `HEIGHT_RULING` in the
generator.

**5. Rigged, one mesh, rigid weights, no quad crosses a joint.** Nineteen joints, named
exactly as ruled: `root, hips, spine, chest, neck, head, beard, shoulder.L/R, elbow.L/R,
hand.L/R, hip.L/R, knee.L/R, foot.L/R`. One mesh, one material, one palette image, one
primitive, one draw call — all survive, as §A promised.

Three properties are checked **against the written GLB**, not against the mesher's intent:

- *no quad crosses a joint* — every group of four consecutive vertices carries one
  `JOINTS_0` value. It holds by construction, because the mask a greedy run merges over is
  keyed by `(colour, joint)` rather than by colour, but it is checked anyway: a rule that is
  only true by construction is one refactor from being false. **What the rule costs, measured
  rather than guessed:** meshing this body with every voxel forced onto one joint gives 5,345
  quads; meshing it with the real joint assignment gives 5,529. **184 greedy runs are broken
  at a joint boundary** — 3.4% more quads, and the reason an arm can bend instead of shear;
- *rigid weights* — every vertex 1.0 to a single joint, zero in the other three slots;
- *joint names* — as a set, with duplicates caught separately. The exporter emits them in
  its own hierarchy order (`root, hips, hip.L, knee.L, foot.L, hip.R, ...`), and a clip binds
  by name, so order is not the contract and the first version of this check was wrong to say
  it was.

The swing pose is delivered as its own GLB and as five lit renders. It is the same rig bent
by its own bones — `render_dwarf.py` imports the generator's `build()`, so the swing views
cannot be a second model posed by hand.

**6. The flame emits, and nothing else does.** Details in §3.

**7. The zoom strip is committed** as `dwarf-zoom-strip.png`: the lit front view downscaled
nearest-neighbour to 10, 30, 100 and 700 px, each blown back up by an integer factor with the
true-size frame inset at the bottom left of its panel. Answers in §4.

---

## 3. Three decisions with costs, stated rather than buried

### The emissive flame costs a second UV set, not a second image

glTF expresses emission as `emissiveFactor x emissiveTexture`, so per-texel emission needs a
texture that is **black wherever nothing glows** — and the base-colour atlas is not. The two
ways out are a second image (which would have cost the one-image clause) or a **second UV
set** pointing into the same atlas, where every non-emitting quad samples a black cell.

The second UV set is what shipped. `TEXCOORD_1` sends the lantern's pane quads to the flame
cells and everything else to cell 63, which is left black.
`emissiveTexture.texCoord = 1`, `emissiveFactor = [1,1,1]`, emission strength held at
**exactly 1.0** so the exporter does not write `KHR_materials_emissive_strength` and the
asset stays extension-free. Checked on the artifact: `TEXCOORD_1` carries 12 distinct values —
the four corners of the black mask cell and of the two flame cells — and **96 of 22,116
vertices (0.43%) sample a lit cell**. The panes and nothing else.

A second bug the check earned its keep on: the hot-core cell was first placed at the lantern's
*centre*, inside the glass volume. A voxel buried under another voxel has no exposed face, so
it ships no triangle, reaches no render and reaches no UV — the cell existed in the atlas and
appeared in **none** of the five views. It is now on the outer skin of each pane.

**Cost: 176,928 bytes**, the whole `TEXCOORD_1` accessor.

One bug worth recording because it is the kind that ships silently: the first check of this
read the mask cell's UV box straight out of `cell_uv()` and reported that *every* vertex
glowed. glTF's UV origin is the top left and Blender's is the bottom left, so the exporter
writes `v_gltf = 1 - v_blender`; the check was reading the wrong corner of the atlas.

### The rig costs 442,320 bytes, and the file grew while the triangle count fell

| | r2 | r3 |
|---|---|---|
| triangles | 14,398 | **11,058** |
| vertices | 28,796 | **22,116** |
| palette cells | 10 | **23** |
| file | 986 KiB | **1,366 KiB** |

The triangle count **fell 23%** and that is a direct consequence of ask 1. r2's own note was
right that "a rivet or a seam is a COLOUR BOUNDARY, and a colour boundary breaks the
co-planar run that greedy meshing would otherwise swallow" — the grooves were buying
triangles. Removing them lets big co-planar runs merge again. Triangle count is a symptom
here and not a budget, and it moved in the direction the round asked for.

The file still grew, and every byte of the growth is nameable:

```
POSITION    265,392      NORMAL      265,392      TEXCOORD_0  176,928
TEXCOORD_1  176,928  <-- the emissive mask
JOINTS_0     88,464  <-- the rig
WEIGHTS_0   353,856  <-- the rig
INDICES      66,348      total 1,393,308 of a 1,399,040-byte file
```

`WEIGHTS_0` is four floats a vertex to carry a number that is always 1.0 in slot 0 — 354 KB
to say "rigid". Compressing that needs an extension, and this contract has none.

### Two things the generator now owns that it used to import

`dwarf_miner.py` still imports `voxel_pine.py` for the material scaffolding, the GLB reader,
the PNG decoder and the volume oracle, and **still does not edit it**. Two things moved into
the dwarf's own file, both because the round's rules require it rather than for taste:

- **the mesher**, because greedy runs must break at a joint boundary, and because the dwarf
  needs the second UV set and the 8 px cell grid. The pine's half-voxel lattice shift and
  r2's undo of it are both simply absent now — they cancelled exactly, so the vertex
  positions are unchanged;
- **the exporter**, because it has to select the armature as well as the mesh, pass
  `export_skins=True`, and pass `export_rest_position_armature=False` for the posed GLB so
  the pose reaches the node transforms.

---

## 4. The zoom strip, and what it says

`dwarf-zoom-strip.png`, four panels left to right: 10, 30, 100, 700 px tall.

**Does the silhouette read as a bearded dwarf with a lantern at 10 px?** Partly, and
honestly: at 10 px the dark hair mass over a lighter face over a green body over dark boots
reads as a stocky bearded figure, and the pickaxe haft reads as a vertical line beside him.
**The lantern does not** — at 10 px it is two or three pixels of warm colour at the hem line
and could be anything. That is a genuine limit of a 0.33x-height prop at 10 px and not
something more modelling fixes; if the lantern has to read at the wide end it wants to be
bigger or to carry a light, and the light is engine work.

**Does any surface detail turn to speckle at 10 px?** No. Every panel is flat blocks of
value; nothing alternates pixel-to-pixel. This is the measurable difference from r2: a
1-voxel groove at a 3-voxel stride is a 0.33-cycle-per-pixel pattern at this framing, which
is exactly the band that aliases and shimmers under camera motion. The value steps that
replaced it are 4-20 voxels across and downsample into a solid.

At **30 px** the read is unambiguous — beard, belt with buckle, lantern, pickaxe. That is a
useful number for the roadmap: it is roughly the size at which this asset stops losing
information.

---

## 5. `check_asset.py` — it does NOT reject the rig, and that is the finding

The brief predicted the checker would fail on a one-mesh/node clause. **It does not. It exits
0 on both candidates.** Verbatim:

```
> python scripts/bench/check_asset.py src-assets/candidates/SM_VoxelDwarf_Miner01_r3.glb
FIGURES src-assets\candidates\SM_VoxelDwarf_Miner01_r3.glb size_m=1.2x1.2x0.8 min_y_m=0.000000
centre_x_m=0.000000 centre_z_m=0.000000 palette=#474B41,#A9B2AC,#707572,#AF8765
tris=11058 verts=22116
EXIT=0

> python scripts/bench/check_asset.py src-assets/candidates/SM_VoxelDwarf_Miner01_r3_swing.glb
FIGURES src-assets\candidates\SM_VoxelDwarf_Miner01_r3_swing.glb size_m=1.2x1.2x0.8 min_y_m=0.000000
centre_x_m=0.000000 centre_z_m=0.000000 palette=#474B41,#A9B2AC,#707572,#AF8765
tris=11058 verts=22116
EXIT=0
```

Two things follow, and the second is worse than the rejection the brief was braced for.

**There is no one-node clause.** `contract_data()` requires exactly one mesh, one material
and one image, and a skin adds joint **nodes** — the count of which nothing checks. The node
clauses that do exist (`has_applied_transform`, `ancestor_nodes`) are satisfied: the mesh
node and the armature node both carry identity transforms. So a rigged asset passes, silently
and today.

**The palette figure is now wrong, and it is wrong quietly.** It reports **4 of 23 cells**.
The mechanism is exact and reproducible: `palette_from_glb()` samples a 4x4 grid of 16 px
cells at pixel centres `(col*16+8, row*16+8)`. Against an 8x8 grid of 8 px cells, those
sixteen sample points land on this atlas's row 1, columns 1/3/5/7 — palette indices 9, 11, 13
and 15 — and then on twelve unpainted cells, which the trailing-black trim removes. Four
values, no hole, exit 0.

That is precisely the defect `palette_from_glb`'s own docstring was written against:

> "under-reporting is worse than rejecting: it cannot say 'and nothing else is here', and a
> reader comparing seven colours has no way to know three more exist."

It now under-reports 19 colours instead of 3, and the four it does name are not a meaningful
subset — they are an artifact of where the sample grid happens to fall.

**I did not touch the checker**, and I did not lay the atlas out to make its sample grid land
on painted cells. Arranging 16 of 23 colours under those sixteen points would have made it
print a plausible-looking palette line that was still wrong, which is worse than a visibly
broken one; the brief's words for that are the right ones — *a workaround smuggled in from
the art seat is how a contract quietly stops meaning anything.*

**What the V2 profile is owed on the other side**, from this round's evidence:

1. a **node/skin clause** that says what it means — "a skin is permitted, its joints must be
   named from the standing list, weights must be rigid" — rather than passing by omission;
2. the **atlas cell geometry read from the asset**, not assumed to be 4x4x16. The image is
   64x64 by contract; the cell pitch is not, and now demonstrably varies by family;
3. the **revision clause** of §C.4: assert every internal name carries the same `rN` and that
   it matches the generator's constant. The generator asserts this about itself already
   (`carries_revision()`), so the checker has a shape to copy;
4. the **naming clause and the slot.** Today `path.stem != mesh_name` fails, which is why the
   candidate carries `_r3` in its basename and passes. On promotion to
   `assets/gltf/SM_VoxelDwarf_Miner01.glb` the stem loses the revision and the mesh keeps it,
   and the checker will fail — correctly, under a rule that has not caught up with §C.1.

---

## 6. Hundreds of variations — what is real now, and what is design

**Sockets are declared and stable.** `SOCKETS` names four anchor voxel coordinates, and the
four swappable parts are authored against them and nothing else: `beard` (0, 4, 74), `hair`
(0, 0, 63), `hand.R` (31, 2, 33) and `hand.L` (-31, 2, 33). The pickaxe and the lantern are
built from their hand socket and weighted entirely to that hand, so splitting them into their
own assets later is a file move rather than a remodel.

The remaining groups — torso, legs, belt, pack — are still absolute. They are not swap axes
yet, and giving them sockets before there is a second part to mount would be a plugin system
with one plug, which this repo's rules forbid and which would be the wrong kind of work here.

**Beard and hair are removable, and that is CHECKED rather than claimed.** Wolf named them as
the first swap axes, so the brief's phrasing — *"shaped so that removing it leaves a complete
head underneath: no skin voxels borrowed from the beard's volume, no hairline that only works
with this hair"* — is implemented as a falsifiable property. `build_voxels()` takes
`beard=None` / `hair=None`, and every build runs the body twice more, once without each part,
and requires that **no voxel outside the part goes missing or changes colour.**

It is verified by sabotage too: reverting the hair's face window to `FACE_Z` — which is the
coupling below — reproduces the finding exactly, `42` voxels differing, first at `(-3, 19, 71)`.

It found three real defects that a render would not have shown:

- the head's own **mouth** was being painted onto the beard's front surface, so a shorter
  beard would have left a head with no mouth. It is now on the skull, and the mounted beard
  cuts its own over the top;
- the **nose** sat one voxel further forward when the hair was mounted, because a hair voxel
  at z 70-71 was standing in front of the skull and the nose builds onto whatever is
  frontmost. The hair's face window now reaches down to the skull base;
- the **cheek** pass ran off the hairline at z 85, because `range(*FACE_Z)` stops one row
  short of its top.

Mechanically: the skull is a complete skin head to z 88; the hair is placed **only into air**,
never over skin, giving a one-voxel shell around the cranium and a solid cap above 88; the
beard's moustache and sideburns are painted *proud* of the skull and do not replace it.
Removing both parts leaves a bald, beardless, complete dwarf.

**A variant is data, not a file.** The three axes, cheapest first:

1. **Palette swap — free.** The geometry is untouched; a variant is a table of 23 hexes
   against fixed indices. The atlas PNG is **188 bytes**. `PALETTE` is ordered so a
   material's cells sit together, so "a different tunic" is three adjacent entries.
2. **Part swap.** `BEARDS` and `HAIRS` are dicts of builder functions keyed by name, selected
   by `--beard` / `--hair`. One entry each this round; the second is a function with the same
   signature and a new key, and the socket makes it fit by construction.
3. **Scalars** — overall height, girth, beard length index. **Designed, not cut.** Height is
   constrained by §2.4 (N must divide 96), so it is not the free axis it looks like; girth and
   beard length are profile-table scalings and are cheap when there is a second dwarf to want
   them.

**The roadmap limit still stands and this round does not design around it.** The client loads
one GLB for `EntityKind::Dwarf` and hands every dwarf the same material, so live palette
variants need per-instance materials. Nothing here needs an offline assembly step: a variant
is a part list plus a table of hexes, which is data the client could pick at spawn.

---

## 7. Findings, and two are for Wolf to rule on

**A. The mid-body was re-proportioned, and it is the round's one band change.** Measured off
the mp4 at t=9 s (the last full frame; the file is 10.0 s at 24 fps, so `-ss 10` returns
nothing) against its own ~10.3 px voxel pitch: the reference's **belt centres at z 40**, its
**tunic hem lands at z 29**, and trouser shows from there down to a boot cuff at z 16. r2 had
the belt at z 20-26 and the hem at z 19 — the belt sat *on* the hem, there was no skirt, no
trouser showed at all, and the tunic's hem band had nowhere to be. The boots already agreed
with the reference and did not move.

I made the change because ask 2 is otherwise invisible, and I am flagging it because these
are bands the notes record as *measured from the approved reference sheet's front
orthographic*. **The sheet and the mp4 disagree here and I resolved it to the mp4**, on the
brief's own statement that the mp4 is this round's authority. If that is wrong, it is one
table and two constants to put back.

**B. The beard is 18 voxels shorter than r2's, same reason.** r2's ran from the cheeks to
z 30 — 46 of 96 rows — and covered the belt, the whole tunic front and most of the face. The
reference's tip sits at about 47% of the figure's height, which is z 51 here. The first r3
render is the evidence: with the long beard, three greens and a stepped hem were behind it.

**C. The figure is still proportionally head-heavy against the mp4, and I did not chase it.**
`dwarf-vs-dwarf-mp4.png` shows it: our head-and-hair mass is a larger fraction of the height
and our legs a smaller one than the reference's. Fixing it means re-deriving every Z band
from the video rather than from the approved sheet, which is a bigger decision than this round
was asked to take. Named here rather than silently half-corrected.

**D. The tunic reads cooler than the reference's.** Ours is the ruled `#5F7A6A` and its two
hue-preserving steps; the mp4's tunic is a warmer, more saturated green. The ruled hex is not
mine to move — flagging it as the most likely single-value change if the look is still off.

**E. `scripts/gate.sh` cannot run on this seat, and did not.** There is no `cargo` and no
`rustc` on this machine, and `python3` resolves to the Windows Store stub. What I could run
with the repo's own venv:

- `scripts/tests` — **53 tests, 1 error, 6 skipped.** The error is
  `test_resolution_bench` failing to import, because it needs the POSIX-only `resource`
  module. Platform, not regression. `test_check_asset.py` alone: **13 tests, OK**;
- `_bmad/scripts/tests` — 78 tests, 10 errors, all `UnicodeDecodeError` reading ledger files
  as UTF-8 under a cp1252 default. Platform, not regression;
- `scripts/audit-mutations.py` — **533 rows, every literal still matches its target.**

This round changed no Rust and wrote nothing outside `src-assets/`, so the Rust tiers of the
gate cannot have been moved by it — but **that is reasoning, not a green gate, and I am not
reporting one.** Someone on the forge side should run `scripts/gate.sh` over this branch.

**F. Two `textures` entries, one `images` entry.** Base colour and emission are separate
glTF texture objects because they use different `texCoord` indices; both point at
`source: 0`, so the one-image clause is genuinely satisfied and not merely passing.

---

## 8. Cost

`session_tokens.py` in print mode, verbatim. Model read from the session banner:
**Opus 5 (1M context)**, `claude-opus-5[1m]`. Row **`dev-art`**.

```
> python _bmad/scripts/session_tokens.py --transcript <this session>.jsonl
Session token cost  (04a8c15f-8c32-4a46-93c1-e06231d7d273.jsonl, tool=claude)  (241 turns, claude-opus-5)
  input (fresh)            482
  cache creation       718,694
  cache read        62,586,615
  output               542,490
  total processed   63,848,281
  wall-clock            59 min  (elapsed, includes idle gaps)
  est. cost             $49.35  (benchmark - verify rates in PRICES)
```

**`--transcript` had to be passed explicitly, and that is a defect worth someone's time.**
`_claude_project_dir()` builds the slug by string surgery that assumes a POSIX path, so on
Windows it looks for `C:\Users\suihk/.claude/projects/D:\Workspace\frostvein` — a path that
cannot exist — and reports `no transcript found` rather than failing. The real directory is
`~/.claude/projects/D--Workspace-frostvein/`. It is a forge-process `FILE` shared with the
sibling projects, so this is not only the art seat's problem; I did not touch it, because it
lives outside `src-assets/`.

The figure is the WHOLE session, and the session is the whole round: reading the brief and
the existing pipeline, the generator rewrite, ten or so build-render-look iterations, the
sabotage runs, the proofs and this report. There is no earlier row to delta against on this
seat — no cursor file exists here — so nothing is double-counted and nothing is missing.
