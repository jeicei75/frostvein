# Resolution decision for 10.3

Adopt **1.6 metres per simulation cell**, **0.4 metres per terrain visual voxel**, and **visual
subdivision k=4**. Keep the simulation grid at k=1.

The real-world k=4 bench measured 978,272 exposed fine faces, 713,723 greedy quads, 1,427,446
triangles, 64 chunks, 1.655 seconds mesh build time and 157,372,416 bytes peak RSS. k=8 remained
measurable (5,615,092 triangles; 5.203 seconds; 327,237,632 bytes) but is a vehicle candidate,
not the default contract, until gingerspice supplies the required fps readings. The offline bench
wall is k=16 at its 4,000,000-fine-face safety limit.

This deliberately does **not** settle the reference sheet's 16 voxels/cell target. It gives 10.3 a
served terrain number now while leaving trees and five dwarves much finer on their separate asset
budgets.
