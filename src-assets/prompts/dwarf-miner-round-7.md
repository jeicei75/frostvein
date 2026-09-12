# Round 7 — build a face, not a slab

**The style limits are lifted. Build the dwarf the reference actually shows, by whatever means
gets you there, inside three limits.**

Six rounds have produced a figure whose head, measured down the centre of the face, is **three flat
colours**: hair `(32,23,16)` for 12 % of figure height, face `(159,144,129)` for 10 %, beard
`(61,45,32)`. No forehead. Brows merged into the hair. No mouth. The nose is one undifferentiated
strip. That is not a modelling failure — it is what the contract permitted. Every face got one flat
colour from a 16-cell palette, so a forehead, a brow line, an eye white and a mouth were not *hard*,
they were **unrepresentable**.

Wolf's ruling, 2026-09-12:

> *"we need to redo the prompt and start from the scratch just by relaxing limitations like requires
> flat faces, flat shading, no bevels, no subdivision, no smooth normals … only limitations should be
> strict topology quality for game model and triangle budget and yes maybe palette … and then try to
> achieve really what is in reference sheets and shots taken from video as extra reference"*

So the style clauses that have governed rounds 1–6 are **gone**, and they do not come back by the
side door as guidance. What remains is in §1.

---

## 1. The three limits — and they are the only ones

**1. Triangle budget: 4,000 triangles for LOD0.** Counted properly this time (see §6). Round 5 spent
1,116, so there is 3.5x headroom. This is a ceiling, not a target — do not spend it for its own sake.

**2. Topology quality, for a mesh that will be rigged and shipped.**

**Enforced by the exporter — each of these fails the build, and each has been mutation-tested:**

- **Triangles and quads only.** No n-gons.
- **Manifold:** no edge shared by more than two faces, no loose vertices, no loose edges.
- **No degenerate faces** — zero area, or vertices at the same position.
- **Consistent winding** — two faces sharing an edge must traverse it in opposite directions.
- **A UV layer exists.**
- **No unapplied modifiers.** Subdivision and bevel are allowed, but **apply them**: the exporter
  applies modifiers on the way out, so an unapplied one makes the triangle count a lie.

**Required, but judged in review rather than by machine** — no cheap test for these is honest, and a
gate that cannot detect its subject certifies rather than catches:

- **No interior geometry.** Nothing sealed inside the figure that no camera can ever see.
- **UV islands do not overlap**, except for deliberate mirrored pairs.
- **Edge loops where the rig's joints go** — see §7. A joint that has to bend through a single
  rigid quad cannot deform.

**3. One material, one texture map.** See §2. The 16-cell flat palette is retired for this asset.

**Everything else is yours.** Bevels, subdivision, smooth normals, rotated boxes, non-box shapes,
a fine grid, no grid — all permitted, none required. There is no axis-aligned clause, no
flat-and-planar clause, and no silhouette-step metric. The exporter's `check_flat_and_planar` gate
has been removed on the orchestrator's side.

**Style guidance, offered and not enforced:** the reference's own 3D interpretation
(`src-assets/references/dwarf-frames/`) is hard-edged and flat-faced, and it sits in a world of
voxel pines. Departing far from that is a judgement call you may make — **say so in the report and
show it**, so the choice is visible rather than accidental.

## 2. The texture is the point of this round

The face features Wolf named are between **1.4 % and 5 % of figure height**. As geometry an eye
would need cubes at 1.4 % of height — a ~70-cube-tall dwarf, far past any budget. As texture they
cost nothing: the reference carries a forehead, brow, eyes, nose, moustache and mouth in **26 source
pixels** on a figure 147 px tall.

- **One texture map, 256 x 256**, one material, one image datablock named `r7`.
- **Start from the approved palette as the base colours** — `#E9D2BB, #5E4632, #FFFFFF, #5F7A6A,
  #474B41, #A9B2AC, #8B6B50, #6B5B49` — then **paint the values the reference paints**: brow shadow,
  eye socket, eye white, nose highlight and its shadowed sides, moustache, mouth, cheek. The palette
  is a starting point now, not a ceiling.
- **No smooth gradients across a whole part.** The reference's value steps are crisp; keep them crisp.
- **Texel density must be even across the figure.** The face may have its own denser island — say
  what density you gave it.
- Backface culling on. Specular IOR Level stays at its `0.5` default: a `0` emits
  `KHR_materials_specular` and the no-extensions clause rejects the export.

## 3. What must be present and readable — this is the round's acceptance list

Wolf's own list, and each of these is judged by eye against the reference:

1. **A forehead** — visible skin between the hairline and the brows.
2. **Brows that are separate from the hair**, not merged into it.
3. **Eyes with a white and a dark pupil**, set in a socket.
4. **A nose with form** — a lit front and shaded sides, not one flat strip.
5. **A moustache AND a mouth.**
6. **A beard that tapers and steps**, not a rectangle.
7. **Ears.**
8. **A head that is not oversized** — r5's head overflows the crop box the reference's head fits
   inside. Measure head height against figure height on `front.png` and match it.

**Each must still read at 100 px and at 60 px figure height.** Render the strip and look at it. A
feature that vanishes at 60 px is a feature the game will never show.

## 4. The references — all three, and their known defects

- **`src-assets/references/reference-sheet.jpg`** — the upstream sheet, the palette and gear authority.
- **`src-assets/references/dwarf-ortho/`** — 5x nearest-neighbour crops for proportion and depth.
  **Read its README first.**
- **`src-assets/references/dwarf-frames/`** — **NEW AUTHORITY THIS ROUND.** Video frames of the
  reference's own 3D interpretation. `f104` is the clearest near-front face; `f088` is the depth
  authority. Use these for how the character reads **lit and in three dimensions**, which no
  orthographic crop can show.

**Known defects in these documents. Do not rediscover them, and do not obey them:**

- **Check 4 (silhouette step density) is WITHDRAWN.** Round 6 reported placing the pickaxe five
  times to move that number and segmenting a tool handle purely to feed it. A scalar that noise can
  satisfy will be satisfied by noise. Do not measure it; it is not a target.
- **Check 5 (the bare neck) is WITHDRAWN pending re-measurement.** Round 6 reported that the skin
  column at `z/H 0.793–0.679` is **the ear**, not a neck — the sheet's own table puts the ear at
  rows 28–45 — and that the check is unbuildable as written, since head mass ends at `0.707` and the
  shoulder line sits at `0.700`, one row apart. That objection is unresolved. **Build the neck the
  reference shows you and ignore the numbers in check 5.**
- **The pickaxe has four conflicting authorities** — the printed label says head-span/length `0.89`,
  the round-6 brief said `0.54`, `gear.png` measures `0.875`, `front.png` measures `0.38`. A 2.3x
  spread, with no faithful answer. **Choose from the art, state which view you measured, and move on.**
- **`gear.png` and `front.png` are at different scales** with no stated conversion, so a gear
  dimension quoted as a fraction of figure height cannot be verified across them.
- **The ortho crops contain the sheet's own 1-px dimension rules**, on the crown row and the sole
  row. Any automatic silhouette read swallows them as figure. Exclude them.
- **`dwarf-model-sheet.md` keeps a superseded video-derived table ABOVE the authoritative
  orthographic one.** Round 5 partly read the wrong one. The orthographic numbers win.
- **The sheet's printed annotations are unreliable in general** — one reads `0.6x dwarf height` for
  the dwarf's own height. **The drawings are orthographic and trustworthy; the numbers beside them
  are not.**

**Where an input contradicts the art, follow the art and write down what you did.** Rounds 4, 5 and 6
each found real defects this way; three of them are listed above.

## 5. It must be watchable — unchanged, and still a hard requirement

- **Author INSIDE Wolf's running Blender through the MCP addon. Never in a `blender --background`
  subprocess.** Only the final export runs headless.
- **Prove it before your first edit:** query the live scene and report what was already in it — the
  objects present, the current filepath. A subprocess starts from an empty or default scene. **If you
  cannot reach the live instance, stop and say so.** Do not fall back to writing a `.blend` from a
  subprocess: Blender never reloads a file that changed underneath it, so the work would be invisible.
- **Geometry on screen within minutes.** Block the whole figure out first from front-view numbers,
  then refine. Thirty minutes with nothing on screen means you are doing it wrong.
- **One part per tool call**, so parts appear one at a time. Name every object for the outliner —
  `r7_head`, `r7_brow_L`, `r7_beard_lower`.
- **Edit the objects in the scene.** A full wipe-and-rebuild is allowed but is the exception:
  announce it in the report, with the reason, each time.

**MODEL FIRST, SCRIPT LATER — and this round there is NO GENERATOR SCRIPT.** Wolf's call,
2026-09-12. Rounds 4 and 5 both reported the same gravity pulling the session back toward a builder
script, and round 5 ended up re-running one that wiped and rebuilt the whole figure in forty seconds
from a spec file. That is fast, unwatchable, and it makes the `.blend` disposable.

- **The `.blend` IS the deliverable.** It accumulates. Do not write a `dwarf_r7.py` that rebuilds the
  figure, and do not keep the part specs in a file as the real source with the scene as its output.
- **To be explicit, because the MCP addon runs Python either way:** this is not a ban on the addon,
  which is the only way you can reach the scene at all. It is a ban on the *artifact* — **each call
  performs one modelling operation on the live scene**, rather than defining or re-running a builder.
- **Why:** a generator makes you think in loops and parameters, which produces uniform, symmetric,
  procedural form — and "flat blocky stick" has been the verdict since round 4. The features this
  round is judged on are hand-placed and asymmetric. The texture map is hand-painted and could never
  be generated anyway.
- **The byte-identical cold-run regeneration rule died at round 4** and does not come back. The
  finishing condition is that the export is repeatable from the committed `.blend`.

**The one thing you must NOT defer, because it is expensive to retrofit and free to do now: the part
decomposition.** The project's endpoint is *hand-author the parts, generate the COMBINATIONS* — a
dwarf stored as a seed, roughly twelve discrete axes plus four continuous dials. That layer scripts
**parts**, never vertices, so it needs nothing from a generator — but it does need every swappable
feature to be **its own named object** from the start: beard, hair, hat/hood, pack, belt, boots,
tunic, tools. Build them separate and named and the combination layer is trivial later. Build them
welded into one torso and it is a rebuild.
- **Save the `.blend` after every part**, and write a progress render per part. Four of six delegated
  runs in this project have been killed by the harness; the saves are the crash insurance.
- **Do not stop and wait for approval.** Work through to a complete figure and deliver. Watchable and
  unattended are not in conflict.

## 6. The exporter and the checker

One command, from the repo root:

    blender --background src-assets/blender/SM_VoxelDwarf_Miner01.blend \
            --python src-assets/blender/export_dwarf.py

`REV` is `r7`, so **your collection must be named `SM_VoxelDwarf_Miner01_r7`**. Changed on the
orchestrator's side for this round:

- `check_flat_and_planar` **removed** — rotation, bevels, subdivision and smooth normals all pass.
- **The triangle count is now real.** It was `len(polygons) * 2`, which assumes every face is a quad
  and silently under-reports the moment topology is mixed. It now counts Blender's own loop
  triangles and **fails the build above 4,000**.
- **The topology gate from §1 is enforced**, and its report line names every violation.

Verified before this brief was written, against round 5's real `.blend` and against one deliberate
mutant per clause: the export runs clean (`1116 of 4000`, every topology clause `0`), and n-gons,
non-manifold edges, loose verts, degenerate faces, flipped winding, a missing UV layer and an
over-budget count each fail the build. The report line reads:

    triangles         1116  of 4000 budget
    topology          n-gons 0   non-manifold edges 0   loose verts 0   loose edges 0
                      degenerate faces 0   flipped winding 0   missing UV layer 0
                      unapplied modifiers 0

`check_asset.py` is known-broken in three clauses and **none of them are yours to chase**: its grid
clause will fail a non-lattice model, its palette clause reads only 4 cells of any atlas, and its
naming clause has not been decided for a revisioned mesh. **Report its output verbatim, pass or
fail. Do not change the model to satisfy it and do not edit the checker.**

## 7. Do not foreclose the rig — it is the next round

Joint names are fixed: `root, hips, spine, chest, neck, head, shoulder.L/R, elbow.L/R, hand.L/R,
hip.L/R, knee.L/R, foot.L/R`, plus `beard`. **Put edge loops at every one of those joints.** Build a
neutral standing base — **stance belongs to the rig, not the mesh**, and that ruling has not changed.
The sheet's arm span of `0.772 H` is a property of its *pose*, not of the body; do not chase it.

## 8. Judgement renders must be lit

Round 5's "lit" pass produces **identical RGB across 10 % of the figure** — it adds no form at all,
and every look decision so far was made under it. Copy `render_r5.py` to `render_r7.py` and:

- keep the flat pass for palette measurement,
- **add a key-lit pass with a strong directional key and real shadow**, roughly matching `f088`'s
  light direction, which is what the reference's sense of form comes from,
- render at **absolute paths** — Blender resolves a relative `render.filepath` against the `.blend`,
  and round 5's renders escaped to `C:\src-assets\renders\`,
- put the revision in every path: `src-assets/renders/r7/`.

## 9. Deliverables

| # | what | path |
|---|---|---|
| 0 | the live-scene report proving you are in Wolf's Blender | in the report, before any geometry |
| 1 | the source, saved after every part | `src-assets/blender/SM_VoxelDwarf_Miner01.blend` |
| 2 | the texture map | `src-assets/blender/textures/T_VoxelDwarf_r7.png` |
| 3 | the render script | `src-assets/blender/render_r7.py` |
| 4 | the exported GLB (gitignored scratch) | `src-assets/export/SM_VoxelDwarf_Miner01.glb` |
| 5 | progress renders, one per part | `src-assets/renders/r7/progress/NN-<part>.png` |
| 6 | five final views, flat **and** key-lit | `src-assets/renders/r7/` |
| 7 | the readability strip at 100 px and 60 px | `src-assets/renders/r7/readability.png` |
| 8 | side-by-side against `front.png` and `side-left.png` | `src-assets/renders/r7/vs-ortho-*.png` |
| 9 | a face close-up beside `f104`, matched scale | `src-assets/renders/r7/vs-face.png` |
| 10 | your report, including every §3 item and how you built it | `src-assets/prompts/dwarf-miner-round-7-report.md` |
| 11 | the exporter's output and the checker's output, verbatim | in the report |
| 12 | your cost — `session_tokens.py` print mode, model from your banner, row `dev-art` | in the report |

## 10. How this round is judged

By Wolf's eye against the reference, on the eight items in §3 — **that list is the round**. Plus
these, which are mechanical:

- the export passes: **≤ 4,000 triangles**, topology gate clean, one material, one image, `r7` on
  every datablock
- every §3 feature still readable at 60 px
- the head's height against figure height matches `front.png`
- edge loops exist at all fifteen joint sites
- nothing written outside `src-assets/`, and **no git command run at all** — the operator commits

## 11. What is NOT in this round

Lighting for the game is Epic 11's. The rig and the pose are next round. Decimation and LODs come
after the look is signed off. **The endpoint this builds toward** is unchanged: hand-author the
parts, generate the combinations — a dwarf stored as a seed, expressible as a part list plus a
table of hexes.

**Making that combination layer work in Python is a SEPARATE task, after the look is signed off.**
It is not deferred by accident and it is not yours to start. All this round owes it is the part
decomposition in §5 — separate, named objects — which costs nothing to do while modelling and is a
rebuild to add afterwards. Do not shape the geometry around an imagined variant system beyond that.
