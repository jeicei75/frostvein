# Resolution bench detail rule

The bench and the client both give every fine top-surface sample a **closed downward pit of 0–2
fine voxels**, clamped so a pit can never cut through its cell. Both then emit the vertical faces
that connect columns of different depth — inside a cell and across cell boundaries — so the fine
surface stays a valid closed voxel surface. This makes otherwise flat cell tops stop being one
greedy quad.

This is a measurement stand-in only, not the terrain look. Story 10.4 owns authored appearance.

## One rule, and it is tested

The two sides are one rule only if they agree bit for bit, and they did not: the client
`wrapping_mul`s in u32 while the bench multiplied unbounded Python integers, so the two agreed at
chance for every k>1 while k=1 — where the clamp forces every depth to zero — stayed identical.
Raw offset agreement was measured at 20.4% over 73,960 points, which is exactly chance for a
five-outcome function.

The same vector is now pinned as literals on both sides —
`scripts/tests/test_resolution_bench.py::ResolutionDetailRuleTests` and
`crates/gui/src/project.rs::the_detail_rule_matches_the_benchs_pinned_vector` — so a drift on
either side turns one of them red. Mutation rows sabotage a multiply on each side and both are
killed.

## Seed provenance — stated accurately

The rule is seeded by the literal `0xF005_7E1A`, hardcoded on both sides. **Nothing reads a seed
off the wire**: `protocol` carries no seed field and `export_world.py` emits none, so the earlier
claim that this is "seeded from the world seed (NFR3)" was true only by coincidence of literals.
`gui` cannot import `sim_core::DEFAULT_SEED` — the no-`sim-core`-edge rule the gate enforces
forbids it — and the bench reads a JSON snapshot, not the simulation.

That is acceptable for a measurement stand-in and would not be for authored terrain. If 10.4 wants
the look to follow the world seed, the seed has to reach the client on the wire first.
