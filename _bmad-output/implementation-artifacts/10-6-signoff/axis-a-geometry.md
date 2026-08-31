# Axis A — visual-resolution geometry sweep

Venue: devpod, 2026-08-31. Input: a real loopback export at tick 21 (`128×128×32`,
`7,180,286` bytes). The bench rejects the run before sweeping if the k=1 independent oracle does
not equal 61,142 exposed faces and 19,264 greedy quads.

| k | Exposed fine faces | Greedy quads | Triangles | Chunks | Mesh build | Peak memory |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 61,142 | 19,264 | 38,528 | 64 | 0.735 s | 112,787,456 B |
| 2 | 285,490 | 77,540 | 155,080 | 64 | 0.902 s | 124,715,008 B |
| 4 | 1,417,777 | 498,714 | 997,428 | 64 | 1.870 s | 189,104,128 B |
| 8 | **FAILED** | — | — | — | — | conservative detailed-face limit: 11,739,264 > 4,000,000 |

The sweep reaches a guarded hard limit at k=8; k=4 is the last completed measurement. It stops
before constructing enough Python fine-face objects to destabilise the devpod. The limit is the
offline instrument's safe ceiling, not a claim that a Rust renderer has the same ceiling.

**This ceiling is a VENUE CONSTRAINT, not a measured wall (Wolf, 2026-08-31).** The devpod shares
a WSL host with two other live projects and CPU peaked near 90% during this run, so sweeping until
something genuinely runs out was deliberately deferred to a quiet host. k=8 is known to be
reachable — an earlier run in this same session completed it at 327 MB — so the re-sweep is owed
work, and the adopted k may change when it happens.

## Per-class budgets

| Class | Instances | Recommended bench scale | Geometry budget | Basis |
|---|---:|---:|---:|---|
| Terrain | 44,984 exposed cells | k=4 | 997,428 triangles / 64 chunks | measured whole exported world |
| Trees | ~265 | up to k=16 per visible cube | ≤814,080 triangles for 265 isolated six-face cubes | derived: 265 × 6 × 16² × 2 |
| Dwarves | 5 | 48 voxels tall (0.025 m) | asset-local; not terrain-bound | derived instance count, vehicle verification still required |

The classes do not share one budget. Terrain's whole-world surface is the binding cost; five
dwarves are not comparable to 44,984 terrain instances.
