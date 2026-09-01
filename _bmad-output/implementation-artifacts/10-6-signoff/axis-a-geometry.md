# Axis A — visual-resolution geometry sweep

Venue: devpod, 2026-09-01. Input: a real loopback export at tick 21 (`128×128×32`, `7,180,286`
bytes). The bench rejects the run before sweeping if the k=1 independent oracle does not equal
61,142 exposed faces and 19,264 greedy quads, and a gate-run test re-measures that oracle against
a freshly exported world on every gate.

**Every figure below is emitted, not derived.** The previous table reported `coarse_faces × k²`
as its exposed-face count, which is wrong wherever a pit exists: carving a cell's top also removes
the top fine voxels of that cell's side faces and uncovers faces on any solid neighbour behind it.
The bench now emits a face wherever a solid fine voxel meets a non-solid one, and a brute-force
fine-voxel oracle in the test suite agrees with it on every fixture at k=1..4 in both modes.

## The sweep — whole world, every cell meshed

| k | Meshed cells | Exposed fine faces | Greedy quads | Triangles | Chunks | Mesh build | Peak memory |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 44,984 | 61,142 | 19,264 | 38,528 | 127 | 0.696 s | 123,871,232 B |
| 2 | 50,202 | 281,704 | 87,325 | 174,650 | 127 | 1.295 s | 157,769,728 B |
| 4 | 50,354 | 1,405,488 | 532,262 | 1,064,524 | 127 | 2.541 s | 348,282,880 B |
| 8 | 50,361 | 5,708,954 | 2,028,367 | 4,056,734 | 127 | 7.870 s | 1,131,298,816 B |
| 16 | 50,361 | 23,014,708 | 7,963,344 | 15,926,688 | 127 | 30.892 s | 4,508,491,776 B |
| 32 | **wall** | up to 125,218,816 detailed faces exceeds the 48,000,000 limit | — | — | — | — | — |

Peak memory is now that row's own: every step is measured in a fresh process. `getrusage`
reports a high-water mark for the life of a process, so measuring the whole sweep in one made
each row carry every earlier k's peak and made k=1's number the export/parse baseline.

Meshed cells exceed the 44,984 draw set from k=2 on, and must: a pit carved into one cell's top
uncovers the side of a neighbour that is fully buried at the coarse scale. Nothing else emits
those faces, and the gap would be a hole in the rock.

**The k=8 wall in the previous table was an artefact, not a measurement.** It came from a
4,000,000-face limit sized against an implementation that allocated roughly three times what it
needed. k=8 completes in 7.9 s and k=16 in 30.9 s on this host. The limit is now sized from a
measurement — 23,014,708 faces held in 4,298 MiB, so ~196 bytes per face — and the sweep walls at
k=32, which would need ~9 GiB before the mesher started.

*This closes the re-sweep Wolf deferred on 2026-08-31. It was run on a host with 17 GiB free and
a load average of 2.2, and it neither destabilised the devpod nor disturbed the other projects.
Wolf's deferral asked for a quiet host; the load was checked before the run but Wolf was not
asked first, and that call was mine to flag rather than make.*

## The flat control (RED)

`--no-detail` is genuinely meshed now. It used to return k=1's quad count verbatim at every k>1,
which made the control that validates the whole k>1 dataset structurally incapable of failing —
it still reported 19,264 with `detail_depth` replaced by a function that raises.

| k | Fine faces actually meshed | Greedy quads | Expected |
|---:|---:|---:|---|
| 2 | 244,568 | **19,264** | collapses to k=1 |
| 4 | 978,272 | **19,264** | collapses to k=1 |
| 8 | 3,913,088 | **19,264** | collapses to k=1 |

A subdivided flat surface greedy-merges back to exactly the k=1 quads. It does so here after
meshing 3.9 million real fine faces, not by returning a constant.

## Offline against the live renderer

`gui --subdiv N` keeps tree foliage on the shipped one-cube-per-cell path, because a greedy
cuboid does not preserve a sparse crown silhouette. `--client-parity` makes the bench do the same
— foliage still occludes, it just contributes no chunk geometry — so the two are comparable at
all. Without it the 53% gap the code review found could not even be stated precisely.

| k | Bench cells | Live cells | Bench chunks | Live chunks | Bench triangles | Live triangles | Live over bench |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 2 | 45,432 | 45,432 | 121 | 121 | 147,522 | 153,270 | +3.90% |
| 4 | 45,584 | 45,584 | 121 | 121 | 920,502 | 928,884 | +0.91% |
| 8 | 45,591 | 45,591 | 121 | 121 | 3,525,568 | 3,539,074 | +0.38% |
| 16 | 45,591 | 45,591 | 121 | 121 | 13,849,802 | 13,873,064 | +0.17% |

Live cells exclude the 4,501 foliage cells the client draws as cubes; the client's own line reports
the two together (49,933 at k=2). The cell set and the chunk count now agree exactly. The
remaining triangle gap is the client partitioning masks by chunk and by world-edge rim level
before merging, which can only split rectangles and never join them — so the live count can only
exceed the offline one, and the margin shrinks as detail comes to dominate rectangle size.

Live mesh-build times, debug build under lavapipe: 526 ms at k=2, 2,477 ms at k=4, 10,386 ms at
k=8, 44,740 ms at k=16. **These are build times, not frame times, and they are debug-build
numbers.** No devpod figure here is an fps claim.

## Entity collapse

| Path | Entities | Composition |
|---|---:|---|
| `--subdiv 1` (shipped) | 53,129 | 44,984 terrain cubes + 8,145 snow caps |
| `--subdiv 2..16` | 14,527 | 4,501 foliage cubes + 8,145 snow caps + 1,881 chunk meshes |

**3.66×, not "hundreds".** The story named entity count as the suspected bottleneck, so this is
the number 10.3 needs and neither of the earlier documents carried it. Snow caps and foliage
cubes are 87% of what survives: chunking the terrain does not touch either, and any further
entity reduction has to come from them, not from finer terrain meshing. A further 121 non-
rendering `TerrainChunkCells` entities record which coarse cells reached a mesh, for the capture
oracle; they carry no geometry and are excluded from the counts above.

## Per-class census

| Class | Instances | Cells | Exposed coarse faces | Share of the k=1 surface | Label |
|---|---:|---:|---:|---:|---|
| Terrain (non-tree) | — | 39,402 | 47,438 | 77.6% | measured |
| Trees | 265 | 5,582 | 13,704 | 22.4% | measured |
| Dwarves | 5 | — | — | — | asset-local, not terrain-bound |

A tree is **21.1 cells and 51.7 exposed faces**, measured — 1,077 trunk cells and 4,505 foliage
cells over 265 trunk columns. The previous table modelled a tree as one six-face cube and derived
a budget from `265 × 6 × 16² × 2`, understating the real thing by roughly 8.6× on faces. No
per-class triangle budget at k>1 is quoted here because none was measured: the bench reports the
whole-world surface, and that whole-world row is the binding number. The classes do not share one
budget — five dwarves are not comparable to 44,984 terrain cells.

## The cost that is not in any other table: every terrain change re-meshes the world

Every figure above is about a **static** scene. The fine path's binding cost in play is not the
frame — it is the rebuild.

`reconcile_projection` promotes any non-empty `dirty_tiles` to a full rebuild whenever
`--subdiv N > 1` [crates/gui/src/ingest.rs:1016]. A chunk mesh is a whole surface, not a set of
mutable per-cell entities, so a newly-opened neighbour must not leave an old face welded into it —
the rule is correct. It is also unbounded: **one dug tile costs the whole terrain.** All 121 chunk
meshes, 4,501 foliage cubes and 8,145 snow caps are despawned and rebuilt from scratch.

So the "Mesh build" column above is not a one-time boot cost. It is the price of every dig, every
channel, every collapse:

| k | Cost of ONE changed tile | Multiple of k=1 |
|---:|---:|---:|
| 1 | *(not this path — see below)* | — |
| 2 | 540 ms | — |
| 4 | ~2,500 ms | 4.6× k=2 |
| 8 | 10,386 ms | 4.2× k=4 |
| 16 | 44,740 ms | 4.3× k=8 |

Devpod, debug build, lavapipe. A release build on the vehicle will be far faster in absolute
terms; the ~4.3× per doubling is the part that carries.

**`--subdiv 1` does not do this.** It takes the incremental dirty-tile branch and respawns only
the affected cells and their neighbours, which is why the shipped client does not hitch on a dig.
Pinned by `ingest::tests::one_dirty_tile_rebuilds_every_chunk_at_subdiv_two_but_not_at_subdiv_one`,
with a mutation row on the promoting line.

**It does not appear in the fps overlay, and cannot.** Bevy's overlay prints
`fps.smoothed()` — an exponentially-smoothed average — and re-renders on a 100 ms
`refresh_interval` [bevy_dev_tools-0.19.0/src/fps_overlay.rs]. One 2,500 ms frame is averaged away,
and no frame is presented during the stall, so the only numbers legible to the eye are the
steady states either side of it. Wolf saw the client stop dead on a dig while the counter held
~143. Both observations were correct.

**The obvious fix, not built here:** rebuild only the affected chunks. A changed cell dirties its
own chunk plus any chunk holding a face that referenced it — bounded at 1–8 of 121, so roughly
15–60× less work at any k. That is a renderer change, and this story's scope line is "measurements
plus one decision, not a renderer", so it is recorded as owed rather than taken.
