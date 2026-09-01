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
| 4 | 928,884 | | | pending Wolf vehicle reading |
| 8 | 3,539,074 | | | pending Wolf vehicle reading |

Both rows are wanted. The earlier card carried only k=4 and said k=8 "is not a vehicle candidate"
— which rested on an offline guard that has since been shown to be mis-sized, not on anything
measured about the renderer. `gui --subdiv 8` builds today.

The reference point for the reading: Wolf's 2026-08-31 run measured ~140 fps at k=4 when the
mesher was submitting **1,527,754** triangles — 44% of which were buried inside rock. Corrected
k=4 submits 928,884, so the k=4 rows should come in at or above that ~140 fps. If they do not, the
regression is in the frame, not in the geometry, and is worth a note.

The default decision remains k=4 pending these readings.
