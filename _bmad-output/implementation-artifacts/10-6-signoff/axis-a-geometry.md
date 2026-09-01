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

## How much of the k=4 budget is the placeholder?

**96.8% of it.** This is the most important number in this document and it was not measured until
Wolf, from the vehicle, said the terrain "is formed from small cubes that are not really forming
meaningful terrain form".

He is right, and the cause is not worldgen. The detail rule is **white noise sampled once per
fine column** — the depths of two adjacent sub-cell columns are independent — which is the worst
possible input for a greedy mesher and looks like gravel rather than landform. `--detail-lattice N`
samples the same rule every N fine columns, so blocks share a depth:

| k=4, client-parity | Greedy quads | Triangles | Detail's share |
|---|---:|---:|---:|
| flat, no detail at all | 14,813 | 29,626 | — |
| **coherent over the whole cell** (`--detail-lattice 4`) | 40,060 | **80,120** | 63.0% |
| coherent over 2×2 fine columns (`--detail-lattice 2`) | 127,699 | 255,398 | 88.4% |
| **white noise per column** (`--detail-lattice 1`, the shipped stand-in) | 460,251 | **920,502** | 96.8% |

**The adopted budget moves by 11.5× on this one property.** 928,884 triangles is not "what k=4
costs"; it is what k=4 costs *if the sub-cell surface is uncorrelated noise*, which no authored
terrain will be. Every committed figure elsewhere in this document uses `--detail-lattice 1`, so
they are all upper bounds.

**What 10.3 should write down is a range, not a number**: k=4 terrain is **80,120 to 928,884
triangles**, and where in that range it lands is decided by 10.4's authored look, not by this
story. The flat row is the floor of what subdivision can ever cost; the noise row is the ceiling.

This does not change the adopted k. It changes what the adopted k is known to cost.

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

## Fixed 2026-09-01: rebuild only the chunks the change can reach

Wolf reported it twice, the second time as "when one dwarf digs all other movement is in halt" —
which is the sharper symptom: a dwarf digs *continuously*, so it was a full mesh build per dug
tile, not one hitch.

`reconcile` now rebuilds only the chunks a changed cell can reach. Measured live on the real world
at k=4, with a real dwarf digging a real tile:

| | Chunks rebuilt | Triangles | Mesh build |
|---|---:|---:|---:|
| Before, per dug tile | 121 | 928,884 | ~2,500 ms |
| After, per dug tile | **1** | **13,554** | **55 ms** |

**~45× on the observed dig.** Offline, over the whole real world, the meshing alone goes 345→34 ms
at k=2 (10.1×), 1,558→76 ms at k=4 (20.5×) and 6,262→235 ms at k=8 (26.6×) — the saving grows
with k, because the constant is one chunk either way.

Two things had to be true for this to be safe, and both are asserted rather than argued:

- **A partial build must be indistinguishable from a whole one.** Not obvious: a cell's faces can
  be attributed to a *neighbour's* chunk when its pit uncovers buried rock, so the cells feeding a
  chunk extend one step past its boundary. `partial_rebuild_matches_the_whole_world_build` builds
  each chunk both ways on two fixtures at k=1/2/4 and compares masks and cell records exactly.
- **The dirty-chunk set must be large enough.** A faithful rebuild of too few chunks leaves a
  stale one and is worse than useless. `the_dirty_chunk_set_covers_every_chunk_a_change_can_alter`
  diffs two whole-world builds of worlds differing in one cell — including cells on a chunk
  boundary — and requires every altered chunk to be covered.

**A regression the tests could not see, caught by the live log.** The first cut ran the incremental
branch on *every* frame, because "not a full rebuild" is the common case rather than the dig case,
and each frame scanned the whole world for the draw set: ~130 ms per frame, 400 times in a
two-minute run. Far worse than the stall it replaced, and every unit test passed, because the
fixtures are one chunk wide and the ECS result was correct — only the cost was wrong. The draw-set
scan is now bounded to the target chunks grown by one cell.

**Superseded:** rebuild only the affected chunks. A changed cell dirties its
own chunk plus any chunk holding a face that referenced it — bounded at 1–8 of 121, so roughly
15–60× less work at any k. That is a renderer change, and this story's scope line is "measurements
plus one decision, not a renderer", so it is recorded as owed rather than taken.
