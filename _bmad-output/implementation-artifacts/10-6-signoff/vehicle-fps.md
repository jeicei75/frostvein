# Gingerspice vehicle run — Wolf-owned

The devpod renders but does not clock. Run each plausible visual subdivision on gingerspice and
read sustained fps from the frame-time overlay; do not substitute a devpod frame rate.

## AC7 — READ 2026-09-01 ON A TERRAIN-DRAWING BUILD. BOTH BARS CLEARED AT k=4 AND k=8.

Wolf, gingerspice, 2026-09-01, build `caa8689-dirty` (RTX 4080 Laptop): **k=4 is over 130 fps and
k=8 varies between 100 and 140.** Both clear NFR6's 60 fps working-zoom bar and its 30 fps
full-vista bar with a wide margin. This is the first reading taken on a build that draws its
terrain AND carries painted snow and the incremental dig rebuild — the three changes that voided
every earlier reading.

**Read the margin, not the number.** k=8 submits 3.8x the triangles of k=4 (3,539,058 against
928,884) and its range OVERLAPS k=4's. A frame rate that barely separates across a 3.8x geometry
change is pinned to something other than the scene — almost certainly the panel. So the sound
conclusion is "both k clear both bars comfortably", and NOT "k=8 has X% headroom". k=8's **100 fps
floor** is the most informative single figure here, because it is the one number that moved.

The three earlier readings remain **void**, each for a different reason, each discovered after the
fact. Kept because the pattern is the lesson.

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
| 4 | 928,884 | `caa8689-dirty` | **>130** | **>130** | **PASSES BOTH BARS**; dig cost 5–13 ms |
| 8 | 3,539,058 | `caa8689-dirty` | **100–140** | **100–140** | **PASSES BOTH BARS**; dig cost 38–78 ms |
| 16 | 13,873,474 | `23172f4` | *(unread)* | *(unread)* | boots in 5,266 ms; digs cost 67–187 ms; previous 60–90 reading VOID |

## OBSERVED ON THE VEHICLE 2026-09-01 — the dig rebuild, live (build `caa8689-dirty`)

Wolf ran `gui.exe --subdiv 4` and `--subdiv 8` on gingerspice (RTX 4080 Laptop, NVIDIA 616.56,
Vulkan). **This closes the coverage hole the round-2 review named**: the incremental dig rebuild
had never been observed on the live path by anyone but its author, and no review layer could fire
it (`designations=0 of 0` over 200 ticks on the devpod).

| k | Build | Boot mesh build | Chunks per dig | Per-dig mesh build | Boot triangles |
|---:|---|---:|---|---:|---:|
| 4 | `23172f4` (clean) | **328 ms** | 1–2 | **4–12 ms** | 928,772 |
| 4 | `caa8689-dirty` | 404 ms | 1–2 | 5–13 ms | 928,884 |
| 8 | `caa8689-dirty` | 1,269 ms | 1–2 | 38–78 ms | 3,539,058 |
| 16 | `23172f4` (clean) | **5,266 ms** | 1 | **67–187 ms** (steady ~70–110) | 13,873,474 |

The k=4 rows agree across two builds and two differently-dug worlds (928,772 against 928,884, on
50,113 against 50,085 projected cubes), with entities 6,826 and chunks 121 identical in both — so
the figures are reproducible, not a single lucky run.

Never 121 chunks. Never a whole-world rebuild. Entities 6,826 and chunks 121 at both k, matching
the devpod exactly.

**THE PER-DIG COST CURVE, measured live across three subdivisions.** Per rebuilt chunk, steady
state: **~5 ms at k=4, ~40 ms at k=8, ~70–110 ms at k=16.** The first dig after boot is slower
(186 ms at k=16) and then settles, so read the steady state, not the first sample. In frames at
60 fps that is well under one, roughly two to three, and roughly four to seven — and it is paid on
every dig.

**THE PER-DIG COST IS THE ONLY THING SEPARATING k=4 FROM k=8.** On frame rate both clear both
bars, so fps does not decide this. On digging they differ sharply: at k=4 a dig costs 5–13 ms —
inside a single 60 fps frame, imperceptible. At k=8 it costs 38–78 ms, a 3–5 frame hitch on *every
dig*, and a fortress digs constantly. This is the first evidence separating the two that is about
playing the game rather than counting triangles, and it points the same way as Wolf's adoption
ruling — which was taken before this measurement existed.

**Roughly half the digs rebuild TWO chunks**, not one — the multi-chunk case is the common case,
not an edge. That is the band the round-2 patch pass widened `dirty_chunks` to cover: before the
fix the reach was one cell where a dig reaches two, so digs in exactly this band were the ones
that could leave a stale chunk standing.

**The vehicle is ~6x faster than the devpod on mesh build** (404 ms against 2,477 ms at k=4), so
every devpod build-time figure in [axis-a-geometry.md] should be read as a devpod-debug ceiling.

**The fps for this run was reported separately by Wolf** (>130 at k=4, 100–140 at k=8); the
pasted logs themselves carry mesh-build times and chunk counts only.

**The SHA column earned its keep on first use.** The 13:06 runs self-reported `caa8689-dirty`,
so those figures were not pinned to any commit. The 13:25 runs report a clean `23172f4` and are.
The k=4 figures agree across both, which is why the earlier pair is kept rather than discarded.

**Also observed: the dirty set coalesces multi-tile deltas.** `rebuilt 1 of 1 chunks for 2 changed
tiles` appears at k=4 — two cells changing in one frame still cost one chunk when they share it,
which is what the set-based `dirty_chunks` is for.

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
