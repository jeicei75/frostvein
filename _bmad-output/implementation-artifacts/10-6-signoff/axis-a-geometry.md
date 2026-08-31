# Axis A — visual-resolution geometry sweep

Venue: devpod, 2026-08-31. Input: a real loopback export at tick 21 (`128×128×32`,
`7,180,286` bytes). The bench rejects the run before sweeping if the k=1 independent oracle does
not equal 61,142 exposed faces and 19,264 greedy quads.

| k | Exposed fine faces | Greedy quads | Triangles | Chunks | Mesh build | Peak memory |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 61,142 | 19,264 | 38,528 | 64 | 0.744 s | 112,308,224 B |
| 2 | 244,568 | 184,385 | 368,770 | 64 | 0.918 s | 118,075,392 B |
| 4 | 978,272 | 713,723 | 1,427,446 | 64 | 1.655 s | 157,372,416 B |
| 8 | 3,913,088 | 2,807,546 | 5,615,092 | 64 | 5.203 s | 327,237,632 B |
| 16 | **FAILED** | — | — | — | — | hard face limit: 15,652,352 > 4,000,000 |

The sweep reaches a guarded hard limit at k=16; k=8 is the last completed measurement. It stops
before constructing enough Python fine-face objects to destabilise the devpod. The limit is the
offline instrument's safe ceiling, not a claim that a Rust renderer has the same ceiling.

## Per-class budgets

| Class | Instances | Recommended bench scale | Geometry budget | Basis |
|---|---:|---:|---:|---|
| Terrain | 44,984 exposed cells | k=4 | 1,427,446 triangles / 64 chunks | measured whole exported world |
| Trees | ~265 | up to k=16 per visible cube | ≤814,080 triangles for 265 isolated six-face cubes | derived: 265 × 6 × 16² × 2 |
| Dwarves | 5 | 48 voxels tall (0.025 m) | asset-local; not terrain-bound | derived instance count, vehicle verification still required |

The classes do not share one budget. Terrain's whole-world surface is the binding cost; five
dwarves are not comparable to 44,984 terrain instances.
