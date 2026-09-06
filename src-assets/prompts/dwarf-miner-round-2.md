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

## One fix in `render_dwarf.py`

It renders with **Workbench, which needs `libEGL.so.1`**. That library is absent on the forge
devpod, so the script exits 134 there: the forge can reproduce your *asset* but not your
*renders*, which leaves the committed PNGs as the only evidence of the look. That is precisely the
failure the "the script is the durable record" clause exists to prevent, one level up from where
it was fixed.

Add a **Cycles CPU** path — denoising **off**, which is required in that venue — selected by a flag
or by falling back when Workbench is unavailable, so one script produces comparable frames on both
seats. Keep Workbench as the default where it works; it is deterministic and has no sampler noise,
which is why you chose it.

## And report your cost

As the brief asks: `session_tokens.py` in print mode against your own transcript, pasted verbatim,
the model read from your session banner, and the row labelled **`dev-art`**.
