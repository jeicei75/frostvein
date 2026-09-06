# Round 2 — the detail pass on the 96-voxel dwarf

**Verified here on your GLB, independently — do not redo any of this.** It regenerates
byte-identical from `dwarf_miner.py` on Linux under Blender 5.2.1, `check_asset.py` accepts it at
exit 0 unmodified, `emissiveFactor` and `emissiveTexture` are both absent, there are no
animations, skins or cameras and a single node, and both new palette cells are present
(`#34271C` hair, `#F0A63C` flame). The face reads: brow ledge, eyes with sclera and pupils, nose
bridge, hair separated from the beard by value. That is a good asset.

## The ask: more detail, and specifically NOT more triangles

You are at **12,006 triangles from 156,898 voxels** — 6,003 quads. Greedy meshing is collapsing
very large flat areas, and that cuts both ways: **adding interior voxels costs zero triangles, and
adding surface noise costs a great many while improving nothing.**

Normalised against the shipped pines as `tris / height²`, you are at **1.30** against their
**1.92–4.26**, so the dwarf is still the least surface-complex object in the world and there is
real room. But the count is an **outcome, not a target.** Do not tune toward a number; a session
can reach 20,000 triangles and make the character worse.

## Five features, from holding your `lit` renders beside `dwarf-contact-sheet.jpg`

1. **The beard is one solid mass.** The reference steps it into strands with a parting. It is the
   largest single read on the character and the biggest gap.
2. **The tunic is a flat field.** The reference has a collar at the neck and cuffs at the wrists.
3. **The belt reads as a plain band.** You already carry a 5,919-voxel `belt` group; the reference
   has a distinct buckle plate on it.
4. **The lantern is a block.** The reference has a cage — corner posts and a top ring — with the
   flame visible *between* bars rather than as a flat face.
5. **The boots have no cuff.** The reference has a turned-over top.

## Everything else in the brief holds unchanged

96 voxels at 0.0125 m, 1.20 m tall, even width in X and Z, `min Y = 0`, one mesh / one material /
one primitive, one 64×64 atlas, greedy-meshed unwelded quad soup, **no emissive at any colour**,
`src-assets/` only, and the byte-identical cold-run proof as the finishing condition.

## `render_dwarf.py` — already changed on the forge side, one thing to check

**Do not redo this; pull it.** Two problems were found and fixed there, because neither could be
tested from the art seat:

1. **The script did not run on the forge at all.** Workbench renders through EGL and
   `libEGL.so.1` is absent on the devpod — Blender *aborts* with exit 134 before writing anything.
   `--engine cycles` now renders the same five views on CPU with denoising off. Workbench stays
   the default where it works; there is no auto-fallback, because Workbench does not raise when
   EGL is missing, it kills the process, so there is nothing to catch.

2. **The flat pass was not flat.** Its docstring calls it "the only honest way to read the
   palette", and measured against the renders you committed, **zero of the ten palette hexes
   survive to the PNG** — skin `#E9D2BB` reads back as `#BDB3AA`. The pass is unlit, but the view
   transform still rewrites every colour, so a palette read off that frame is wrong in silence.
   The transform is now set to `Standard` for the flat pass only, and restored for the lit one, so
   the lit look you judged is unchanged.

`assert_flat_is_flat()` now makes the script prove this every run rather than claim it in prose.
It is verified by sabotage on the Cycles path: removing the transform line fails it at exit 1 with
all ten colours reported missing.

**What is owed from your seat, and it is one line of output:** run the script under Workbench as
you normally do and confirm it prints `FLAT-CHECK all 10 palette colours reach the PNG exactly`.
The Workbench half of the fix could not be verified on the forge — no EGL — so you are the only
one who can close it. If it fails, say so with the list it prints; do not work around it.

## And report your cost

As the brief asks: `session_tokens.py` in print mode against your own transcript, pasted verbatim,
the model read from your session banner, and the row labelled **`dev-art`**.
