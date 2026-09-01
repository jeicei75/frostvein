# Gingerspice vehicle run — Wolf-owned

The devpod renders but does not clock. Run each plausible visual subdivision on gingerspice and
read sustained fps from the frame-time overlay; do not substitute a devpod frame rate.

## AC7 IS OPEN. NO VALID READING EXISTS FOR ANY k.

Three fps readings have been taken for this story and **all three are void**, each for a different
reason, each discovered after the fact. That is the finding this card now exists to prevent
repeating.

| Reading | Taken | Voided by | Why |
|---|---|---|---|
| ~140 fps | 2026-08-31 | `741b93d` | Terrain was entirely back-face culled — the scene had no terrain in it |
| 143.24 fps @ k=4 | 2026-09-01 | `741b93d` | Same; and it barely moved across a 39% triangle cut, which was the tell |
| 60–90 fps @ k=16, 4K | 2026-09-01 | `bace455`, `c8675fc` | Taken on a build predating painted snow (−8,145 entities) and the dig rebuild |

**Therefore: record the commit SHA a reading was taken at, in the table, every time.** Two of the
three voidings above would have been caught immediately by that one column. A reading without a
build is not a measurement.

## The commands

```text
gui.exe --subdiv 4          # boot framing, then F3 for the fps overlay
                            # hold E to pull out to the full vista, Q to come back
gui.exe --subdiv 8          # the unread middle candidate — this is the one that shows the ceiling
gui.exe --subdiv 16         # re-take: the previous k=16 reading is void
```

**`--distance` is not usable here.** The card previously read `gui.exe --subdiv 4 --distance 500`,
which exits with `--distance requires --capture`, so AC7's full-vista bar was unobtainable as
written. Zoom is interactive: `E` pulls out and `Q` comes in, clamped to 4.0–500.0 world units
(`ingest.rs:865`), and `F3` toggles the overlay (`ingest.rs:921`).

## The table to fill

Triangle figures are CHUNK MESHES ONLY and exclude the 4,501 foliage cube entities (~54k tri).

| k | Triangles submitted | Build (SHA) | Boot framing fps (NFR6 ≥60) | Full vista fps (NFR6 ≥30) | Result |
|---:|---:|---|---:|---:|---|
| 4 | 928,884 | | | | **adopted k — unread** |
| 8 | 3,539,074 | | | | unread |
| 16 | 13,873,064 | | | | unread; previous reading void |

## What to expect, and what would be suspicious

- **A number pinned at ~144 is the display, not the scene.** The voided k=4 readings sat at
  143.24 and ">140" across large changes in geometry, which is the signature of a refresh-rate
  cap. If two different k values report the same fps, suspect vsync before believing headroom.
- **k=8 is the informative run.** k=4 is likely to be capped and k=16 is the extreme; the middle
  is where the curve becomes legible.
- **Digging hiccups are expected and are not the frame rate.** At k=16 one chunk is ~230 ms of
  mesh build on the devpod's debug build, proportionally less on the vehicle. Wolf: "we can
  optimize later on." Named as owed, not taken.

## Why the earlier readings were void — kept so the pattern is legible

**The winding defect.** The chunk mesher wound every quad to face opposite its own normal, and
`StandardMaterial`'s default `cull_mode: Some(Face::Back)` then discarded the entire terrain
surface. What those runs rendered was snow caps, tree cubes and trunks floating over a void — the
"visible holes" in both of Wolf's reports. The triangles were still submitted and still paid for
vertex processing and culling, but almost nothing shaded a fragment. That is also why fps stayed
at ~140 across a 39% cut in submitted triangles: the cut was in geometry never being rasterised.
Fixed 2026-09-01, with before/after captures beside this file:
`winding-a-subdiv1-shipped.png`, `winding-b-subdiv4-before.png`, `winding-c-subdiv4-after.png`,
all three the same camera on the same world.

**The scene changed underneath the k=16 reading.** `bace455` replaced 8,145 snow-cap slab entities
with paint on the fine terrain's top faces (entities 14,527 → 6,826) and `c8675fc` replaced the
whole-world rebuild with a per-chunk one. Both change what the vehicle draws and what it does on a
dig, and both landed after the reading was recorded.

**k=8 was never excluded on evidence.** The earlier card said k=8 "is not a vehicle candidate",
which rested on an offline guard since shown to be mis-sized, not on anything measured about the
renderer. `gui --subdiv 8` builds today.
