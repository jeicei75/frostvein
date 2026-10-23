# 13.2 scale points

Ten files, crews 5/20/50/100/200 on 128x128x32 and 256x256x32, DEFAULT_SEED, 2000 ticks each. Generated, not
hand-written: apply `census.patch`, `counters.patch`, `scale.patch` on 4f57027, then
`SCALE_OUT=<this dir> SCALE_TICKS=2000 cargo test --release -p sim-core --lib scale_generate -- --ignored --nocapture`
(it also prints the camp component size, #159). Work scales with the crew: 8 dig marks and 1 cut mark per dwarf, and a
stockpile of one cell per mark. Dig marks grow from the diggable cell beside the camp's walkable component nearest
(edge/4, edge/4); cut marks are the trees nearest that point; the pile is the walkable cells nearest (3*edge/4, 3*edge/4).
Camp is at the centre, so mine and pile sit in opposite quadrants and the haul crosses the map (the legs double on 256).
