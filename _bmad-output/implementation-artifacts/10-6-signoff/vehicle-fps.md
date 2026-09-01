# Gingerspice vehicle run — Wolf-owned

The devpod renders but does not clock. Run each plausible visual subdivision on gingerspice and
read sustained fps from the frame-time overlay; do not substitute a devpod frame rate.

```text
gui.exe --subdiv 4          # boot framing, then F3 for the fps overlay
                            # hold E to pull out to the full vista, Q to come back
gui.exe --subdiv 8          # the second candidate, see below
```

**`--distance` is not usable here.** The card previously read `gui.exe --subdiv 4 --distance 500`,
which exits with `--distance requires --capture`, so AC7's full-vista bar was unobtainable as
written. Zoom is interactive: `E` pulls out and `Q` comes in, clamped to 4.0–500.0 world units
(`ingest.rs:865`), and `F3` toggles the overlay (`ingest.rs:921`).

| k | Triangles submitted | Boot framing fps (NFR6 ≥60) | Full vista fps (NFR6 ≥30) | Result |
|---:|---:|---:|---:|---|
| 4 | 928,884 | **>140** (2026-09-01) | | **PASSES NFR6's 60 bar** — but see the cap note below |
| 8 | 3,539,074 | | | pending Wolf vehicle reading |

## The k=4 reading, 2026-09-01 — the first valid one

Wolf: "still over 140fps, no halts anymore." This is the first reading taken of a scene that
actually draws its terrain, and it clears NFR6's 60 fps working-zoom bar with room to spare.

**It is still probably a display cap, so it is a floor and not a measurement of headroom.** 143.24
was read before the winding fix, when the terrain was not rasterised at all; >140 is read after,
with 928,884 triangles actually drawn. A number that barely moves across that change is pinned to
something other than the scene — almost certainly a 144 Hz refresh. So the honest reading is
"k=4 comfortably exceeds 60", not "k=4 has 2.4x headroom". **k=8 is what would show where the
real ceiling is**, and it is still unread.

Full vista at k=4 is also still unread (hold E).

## Both earlier fps readings measured a scene with no terrain in it

**Every `--subdiv N > 1` fps number taken so far is void**, including the ~140 fps of 2026-08-31
and the 143.24 fps of 2026-09-01. The chunk mesher wound every quad to face opposite its own
normal, and `StandardMaterial`'s default `cull_mode: Some(Face::Back)` then discarded the entire
terrain surface. What those runs rendered was snow caps, tree cubes and trunks floating over a
void — the "visible holes" in both reports. The triangles were still submitted and still paid for
vertex processing and culling, but almost nothing shaded a fragment.

That also explains the reading that made no sense: fps stayed at ~140 across a 39% cut in
submitted triangles, because the cut was in geometry that was never being rasterised. It is worth
checking whether 143.24 is simply a 144 Hz vsync cap while reading the new numbers — if the fps
sits at the refresh rate again, the overlay is measuring the display, not the scene, and neither k
has been tested.

Fixed 2026-09-01, with before/after captures beside this file:
`winding-a-subdiv1-shipped.png`, `winding-b-subdiv4-before.png`, `winding-c-subdiv4-after.png`,
all three the same camera on the same world.

Both rows are wanted. The earlier card carried only k=4 and said k=8 "is not a vehicle candidate"
— which rested on an offline guard that has since been shown to be mis-sized, not on anything
measured about the renderer. `gui --subdiv 8` builds today.

There is no usable reference point from the earlier runs, for the reason above. This is the first
reading that will be taken of a scene that actually draws its terrain, so expect it to be LOWER
than 143 rather than higher, and read that as the instrument working rather than a regression.

The default decision remains k=4 pending these readings.
