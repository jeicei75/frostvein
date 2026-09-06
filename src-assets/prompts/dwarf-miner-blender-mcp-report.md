# Session report — the dwarf miner, via Blender MCP

**Answers:** `prompts/dwarf-miner-blender-mcp.md`.
**Date:** 2026-09-06. **Seat:** Windows 11 art seat, Blender **5.2.1 LTS** (build 2026-08-25),
Blender MCP attached. **Status:** delivered; three findings below are Wolf's to rule on.

---

## The three open decisions, as ruled

The brief ended with three decisions reserved for Wolf. All three were taken before any
modelling, on 2026-09-06:

| # | Decision | Ruling |
|---|---|---|
| 1 | The published name | **`SM_VoxelDwarf_Miner01`**, as proposed |
| 2 | Is the gear part of the mesh? | **Bake in, keep separable** — the brief's own recommendation |
| 3 | The 1.20 m dwarf anchor | **Ratified at 1.20 m** |

Ruling 3 was taken **before** the measurement in finding 3 existed. That finding is the
reason to re-hear it; nothing else here depends on re-opening it.

## Deliverables

| Path | What |
|---|---|
| `src-assets/blender/dwarf_miner.py` | the standalone headless generator — **the durable record** |
| `src-assets/blender/dwarf.blend` | the editable source, written by the generator's `--blend` |
| `assets/gltf/SM_VoxelDwarf_Miner01.glb` | the runtime glTF, at the location `docs/tech-art-guidelines.md:463` contracts |
| `src-assets/ASSET_NOTES.md` | a dwarf section: generation, palette, the three findings |

Nothing was committed or pushed. `voxel_pine.py` was **not edited** — the generator imports
it, as the brief required.

---

## 1. The `FIGURES` line, and the SHA-256 from two independent runs

```
FIGURES name=SM_VoxelDwarf_Miner01 voxel=0.100 voxels=239 groups=body:216,lantern:12,pickaxe:11
  quads=108 verts=432 tris=216 bbox=1.000x1.200x0.600 centre_x=+0.000000 centre_z=+0.000000
  min_y=+0.000000 volume=0.239000 expected_volume=0.239000 materials=1 primitives=1 images=1
  palette=#E9D2BB,#5E4632,#FFFFFF,#5F7A6A,#474B41,#A9B2AC,#8B6B50,#6B5B49 glb_bytes=16688
OK SM_VoxelDwarf_Miner01 -> assets/gltf/SM_VoxelDwarf_Miner01.glb
```

Three cold Blender runs into three separate paths — no MCP, no hand steps:

```
3a87cbbd03660316e152c6a75ae959daf4bcdd64aea42301767234ec71dd6760  assets/gltf/SM_VoxelDwarf_Miner01.glb
3a87cbbd03660316e152c6a75ae959daf4bcdd64aea42301767234ec71dd6760  <scratch>/run-a/SM_VoxelDwarf_Miner01.glb
3a87cbbd03660316e152c6a75ae959daf4bcdd64aea42301767234ec71dd6760  <scratch>/run-b/SM_VoxelDwarf_Miner01.glb
```

`cmp` confirms identical, not merely equal-digest.

**There is no `--seed`, and that is not an omission.** Unlike the pines this generator draws
no random numbers at all — every voxel is placed by an explicit rule — so determinism is by
construction rather than by discipline about hashing.

### The self-verification the script carries

Every figure is read back **out of the written GLB**, never off the spec that was asked for.
Beyond the brief's list, the generator also asserts the 0.1 m grid clause, `CLAMP_TO_EDGE`
wrap, the basename/mesh/node naming triple, and the absence of animations, skins, cameras,
extensions and any emissive field.

The two non-optional guards are present and were exercised:

| Invocation | Exit | First line of stderr |
|---|---|---|
| `--voxel 0` | 1 | `error: --voxel must be greater than 0 (got 0.0)` |
| `--voxel -0.1` | 1 | `error: --voxel must be greater than 0 (got -0.1)` |
| wrong basename (`dwarf.glb`) | 1 | `error: basename must be 'SM_VoxelDwarf_Miner01', got 'dwarf.glb'` |
| not a `.glb` | 1 | `error: output must be a .glb path` |
| unknown option `--seed 7` | 1 | `error: unknown option '--seed'` |
| no arguments | 1 | usage |
| `--voxel 0.2` | 0 | — |

The basename guard is an addition to the brief's list: the contract requires
`basename == mesh name == node name` and that string is compiled into the client, so the
generator refuses to write a GLB the checker would reject rather than writing one and failing
afterwards.

## 2. `scripts/bench/check_asset.py`'s verdict

**Accepted. Exit 0. No code change to the checker.**

```
FIGURES assets\gltf\SM_VoxelDwarf_Miner01.glb size_m=1.0x1.2x0.6 min_y_m=0.000000
  centre_x_m=0.000000 centre_z_m=0.000000
  palette=#E9D2BB,#5E4632,#FFFFFF,#5F7A6A,#474B41,#A9B2AC,#8B6B50 tris=216 verts=432
```

The brief anticipated that some V1 clause might prove impossible for a character and asked
for a report rather than a workaround. **None did** — but that outcome was not free, and
finding 2 below is the clause that came closest to breaking. The checker prints seven palette
cells because its reader is sized to the pine's seven-colour atlas; the dwarf ships eight and
the eighth is verified by the generator instead. That is a display limit in the checker, not
a rejection.

## 3. Renders

Five views — front, both orthographic sides, back, and one ¾ — rendered headless from the
**written GLB**, on the neutral studio backdrop `scripts/bench/spike_pine_render.py` uses, so
the frames carry no client scene lighting the asset does not own.

Currently in the session scratchpad: `renders/sheet-vs-renders.png` (Section A above, the
five renders below), `renders/strip.png`, and the five individually. **They have no permanent
home yet** — placing them under `_bmad-output/implementation-artifacts/10-5-signoff/` is
Wolf's call, not an assumption this session should make.

Three cuts were needed before the silhouette read, each recorded in the generator's comments:

1. the pickaxe read as a **hook** — a bare bar at the top of a haft. Fixed by turning both
   ends of the head down, which is the least that still reads as a pickaxe.
2. the pickaxe **bisected him in profile** — it was carried at `y=0`, dead centre in depth.
   Moved to `y=1`, a voxel proud of the torso.
3. the lantern read as **a box on the floor** — slung on a bail at `z 1-3`, then `z 2-4`.
   Its cap now sits level with the fist at `z=5` with the body just under, which is how the
   contact sheet carries it.

## 4. What the budget forced out, and what could not be satisfied

### Finding 1 — `Pants` and `Wood` cannot be settled from the sheet

**Both values ship as the brief's, carried and NOT confirmed.**

The brief asked for a re-read from a full-resolution copy of the sheet. There isn't one:
`reference-sheet.jpg` is **1024 × 558**, and that is its full resolution, not a downscale.

At that size the sheet's `8` and `B` are the same glyph — provably, because `Wood Trunk`
reads `#685849` there while the pines demonstrably ship `#6B5B49`. The brief established
that; this session tried to break the remaining ties by sampling the swatches, and **could
not**.

Absolute luminance is unusable (measured bias across swatches runs −17..+1). Channel
differences cancel most of that, so they were calibrated against the three swatches whose
truth is known from the shipped pine atlas:

| Swatch | sampled R−G / G−B | truth R−G / G−B | error |
|---|---|---|---|
| Snow `#FFFFFF` | +0.52 / +0.78 | 0 / 0 | **+0.52 / +0.78** |
| Wood Trunk `#6B5B49` | +16.93 / +14.36 | +16 / +18 | **+0.93 / −3.64** |
| Needle Green `#364D3F` | −14.18 / +10.70 | −23 / +14 | **+8.82 / −3.30** |

Residual error reaches **8.8/255**. The candidates are **3/255** apart, so the method cannot
discriminate:

| Role | Candidate | predicts R−G / G−B | residual |
|---|---|---|---|
| Pants (sampled −1.53 / +6.49) | `#474B41` *(shipped)* | −4 / +10 | +2.47 / −3.51 |
| | `#474841` | −1 / +7 | **−0.53 / −0.51** |
| Wood (sampled +28.70 / +22.93) | `#8B6B50` *(shipped)* | +32 / +27 | −3.30 / −4.07 |
| | `#8B6850` | +35 / +24 | −6.30 / −1.07 |

The residuals actually lean **`#474841`** for Pants — against the brief's stated
`G > R > B` argument, which both candidates satisfy — and **split** for Wood. Neither lean
clears the noise floor, so neither overturns the brief.

**One look at the original art file settles both.** Nothing available in this repo can.

### Finding 2 — the pine's half-voxel lattice shift is off the project grid at 0.1 m

`voxel_pine.greedy_mesh()` subtracts half a voxel in X and Y so an **odd**-width trunk
centres on the origin. At the pine's 0.2 m voxel that shift is 0.1 m and lands on the grid.
At the dwarf's **0.1 m** voxel it is **0.05 m**, and `check_asset.py`'s grid clause rejects
every vertex.

The dwarf is therefore **even-width in X and Y**, and the shift is undone after meshing. That
is a rigid translation of the whole quad soup, which is exactly why the mesher itself could be
imported **unedited** — the brief's constraint held.

A consequence worth recording as a contract question: the resolution contract's phrasing
*"every voxel centre lands on the 0.1 m lattice"* and the checker's clause *POSITION values
on the 0.1 m grid* **cannot both hold at a 0.1 m voxel**. One is satisfied by the corners,
the other by the centres, and they are half a voxel apart. The checker is the mechanical
gate, so it won here; voxel centres land on the half-grid.

### Finding 3 — Section A depicts roughly twice the resolution the contract allows

The contract fixes the dwarf at 12 voxels of 0.1 m
(`docs/tech-art-guidelines.md:352`, *"dwarves target 0.1 m (12 voxels = 1.20 m = 0.75 cells)"*).

Measured off the sheet's own **back view**, the cleanest standing silhouette on it —
bounding box **29 × 92 px**, edge-step run lengths `{4:4, 5:3, 7:1, 11:1, 2:1}`, modal step
**4–5 px** — the drawn figure is about **20–23 voxels tall and ~7 wide**. Method: isolate the
figure from the blueprint ground by saturation and darkness, then take run lengths of the
left and right silhouette edges; on voxel art every edge step is one voxel. The sample is
small, so the figure is a range, not a number.

It is also **not voxel art**: at 8× the front view shows anti-aliased outlines, gradient
shading, a warm glowing lantern pane and a *curved* organic pickaxe head. Section A is
concept art at illustration fidelity, whatever its title says.

At 12 voxels the following are unavailable — not merely difficult — and were dropped:

- **eyes.** Once hair frames it the face is 2 voxels wide. A separated pair cannot exist, and
  an adjacent pair reads as a brow band rather than as eyes.
- the moustache as a mass distinct from the beard
- bracers; the tunic's layered panels; boot cuffs
- the pickaxe's **curved** pick and its hammer poll — it ships as a straight 3-voxel bar with
  both ends turned down
- the lantern's frame and mullions — it ships as base / pane / cap, three voxels

**Nothing here is fixable by modelling harder.** 0.1 m is the floor the brief and the contract
both set, and a finer voxel is additionally rejected by the checker's grid clause. The exits
are exactly two:

1. **accept the coarse read** — what ships today; or
2. **raise the 1.20 m anchor**, which re-labels every tree's dwarf-multiple on the sheet and
   moves `gui`'s `scale` (already owed a correction to `0.75` by story 10.5).

The model reads as a bearded, gear-carrying dwarf either way. This is a ruling about how much
of the sheet's design survives, not about whether the asset works.

---

## The build, for the record

Axes are Blender's: **+X** is his left, **+Y** is the way he faces, **+Z** is up; the
exporter's `export_yup` lands him facing −Z in glTF.

```
X  -5..4   (10 voxels, 1.00 m)   pickaxe head .. lantern
Y  -3..2   ( 6 voxels, 0.60 m)   backpack ..... beard and nose
Z   0..11  (12 voxels, 1.20 m)   boot sole .... hair and pickaxe head
```

A bounding box is centred only when `max == -min - 1` on an axis, so **the gear is what
balances him**: the pickaxe reaches −5 and the lantern +4, the backpack −3 and the beard +2.
Moving one without the other decentres the asset, and every instance placed from it then
leans the same way. That constraint is load-bearing and is commented as such in the
generator.

Body: boots and trousers `z 0-3` on two columns with a 2-voxel gap; tunic torso `z 4-7` with
a full belt band and a metal buckle on the front face only; backpack on the back plane;
head `z 8-11` as a solid hair block with the face cut back into its front plane, plus a nose
proud of the beard; beard hanging forward over the chest.

**Gear is baked in as separable groups** — `body:216, lantern:12, pickaxe:11`, reported on
every `FIGURES` line. They touch the body only at the hands, and because the mesh is unwelded
quad soup, cutting them out for the mining-strike cycle later is a selection, not a remodel.

### Palette, cell-to-role map

Section A's swatch column with the sheet's `Needle Green` dropped — that one is the tree's.

| Cell | Hex | Role |
|---|---|---|
| 0 | `#E9D2BB` | Skin |
| 1 | `#5E4632` | Beard — also the hair |
| 2 | `#FFFFFF` | Snow — **used here as the lantern's glass pane** |
| 3 | `#5F7A6A` | Tunic |
| 4 | `#474B41` | Pants — *see finding 1* |
| 5 | `#A9B2AC` | Metal |
| 6 | `#8B6B50` | Wood — *see finding 1* |
| 7 | `#6B5B49` | Wood Trunk — belt, boots, backpack |

Cells 8–15 are unused and black, as on the pines.

`Snow` needed a job on a dwarf who is never snowed on. It is the lantern's pane: the one
place underground that wants a pure white, and it reads as glass from all four sides. **It is
plain albedo and not emissive** — the client owns the lantern's light, and a baked emissive
face is a light nobody can switch off. The generator asserts the material carries no emissive
field at all.

Both colour bugs the brief warned about are guarded: the PNG is hand-encoded and packed as
exact bytes rather than via `Image.pack()` on a `GENERATED` image, and the hex is written to
`.pixels` display-referred without linearising. Every cell is decoded back **out of the
finished GLB** and compared.

## What was not run, and what was left alone

- **`scripts/gate.sh` did not run, and no green gate is claimed.** There is no `cargo` on
  this seat — it is the art box, not the build box. Worth knowing before the push: `assets/`
  is in the gate's `CODE_RE` *and* `RENDER_RE`, so the push carrying this GLB will pull the
  **full** gate including the pixel guards.
- `scripts/tests` was run and reports **33 tests, 1 error, 6 skipped**. The error is
  `test_resolution_bench` failing to import `resolution_bench`, which imports `resource` — a
  POSIX-only stdlib module. **Pre-existing platform limitation of this seat, not caused by
  this work**; it passes on the devpod.
- **The pines are undisturbed.** `voxel_pine.py` is unchanged from `HEAD`, and all four trees
  regenerate byte-identical to the shipped files.
- `mise.toml` and `pyproject.toml` are staged as deletions in the index and were before this
  session started. Not touched.
- `dwarf.blend1` is a Blender autosave; `.gitignore` already excludes `*.blend[0-9]`.

## Open for Wolf

1. **Finding 3** — accept the coarse read, or raise the 1.20 m anchor. Ruled once already,
   before the measurement existed.
2. **Finding 1** — `Pants` and `Wood` need the original art file, or a ruling to keep the
   brief's values as shipped.
3. **Finding 2** — the contract's "voxel centres on the lattice" and the checker's "POSITION
   on the grid" disagree at a 0.1 m voxel. The checker won here; the contract's wording may
   want the same correction.
4. Where the renders should live, and whether `assets/gltf/` and the two new
   `src-assets/blender/` files go on a branch now.
