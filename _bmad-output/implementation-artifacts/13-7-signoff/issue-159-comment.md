13.7 creation probes (2026-11-02, `main` `f148cfbe`, release; raw lines in `_bmad-output/implementation-artifacts/13-7-signoff/creation-probes.txt` on branch `story-13-7-the-long-way`).

**Correction to the 13.2 reading above.** 13.2 inferred that most 256² budget-exhausted searches were "reachable goals farther than the budget". Measured directly, none is. Each exhausted search was re-run without a budget to classify it:

- On `scenarios/two-hundred-on-256.json` (256², crew 200, 2,500 ticks), `claim_jobs` ran out of budget 238 times. All 238 goals were **unreachable**.
- `astar()` ran out 915 times, all unreachable: 896 reroutes in `resolve_blocked_step` and 19 re-plans in `execute_jobs`.
- In the fresh 256² valley, no reachable single-goal search from the camp needed more than 14,604 nodes (a sample of 663 of the 64,300 camp-component cells). A search with no reachable goal floods all 64,300 cells, about 35 ms in release.

So the live shape is this issue's title: an unreachable job's search runs out before it finishes, its component is never cached, and only one job is stamped per tick. A better heuristic cannot help, because a search with no goal to find floods everything whatever the heuristic. A budget that a search cannot run out of does fix it.

**Red on the real valley.** 256² DEFAULT_SEED, 5 miners, 25 one-cell sealed pockets with a dig beside each. The 25 digs are designated at t0 and a reachable dig at the camp at t100. On `main` the reachable dig is never attempted by t500, and the tick averages about 102 ms. With the claim budget sized to the world, it is claimed at t115 and dug at t208.
