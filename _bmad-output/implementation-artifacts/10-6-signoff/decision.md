# Resolution decision for 10.3

Adopt **1.6 metres per simulation cell**, **0.4 metres per terrain visual voxel**, and **visual
subdivision k=4**. Keep the simulation grid at k=1.

The real-world k=4 bench measured 1,417,777 exposed fine faces, 498,714 greedy quads, 997,428
triangles, 64 chunks, 1.870 seconds mesh build time and 189,104,128 bytes peak RSS. k=8 is the
first guarded failure (up to 11,739,264 detailed faces), so k=4 is the only vehicle candidate.

**Caveat 10.3 must carry forward:** k=8 was excluded by the bench's own guard on a shared, loaded
host, not by a measured resource wall (Wolf's deferral, 2026-08-31). k=4 is the number to build
against today; it is not proof that k=8 is unservable.

This deliberately does **not** settle the reference sheet's 16 voxels/cell target. It gives 10.3 a
served terrain number now while leaving trees and five dwarves much finer on their separate asset
budgets.
