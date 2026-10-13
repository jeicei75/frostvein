# Story 12.10: Ore in the rock. Run alone, after commit:
#   RUST_TEST_THREADS=1 scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/12-10.sh
# Row names follow the story's Task 6 list.

# Also the ore_wire.py deliberate RED (Verification), run from a scratch-worktree release simd.
mutation "1 dig yields stone whatever the material" sim-core digging_ore_yields_an_ore_item_that_reaches_the_pile <<'PY'
import pathlib
# Re-pointed 2026-10-13 (12.10b): the spawn now lands at `landing` (the target, for a dig); the seam is unchanged.
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        ecs.spawn((Item(yielded), item_id, landing));\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        let _ = yielded;\n        ecs.spawn((Item(ItemKind::Stone), item_id, landing));\n'))
PY

mutation "2 the dig filter refuses ramps again" sim-core a_dig_mark_takes_a_ramp_and_the_ramp_is_dug_out <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                                        terrain.tile(*pos),\n                                        Some(Tile::Solid(_) | Tile::Ramp(_))\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                                        terrain.tile(*pos),\n                                        Some(Tile::Solid(_))\n'))
PY

# 3a is the story's literal row. Ore from another seed of its own consumes no other stream's draws,
# so only the terrain fingerprint can see it (the same-seed/other-seed tests stay green).
mutation "3a ore seeds from STREAM_WORLDGEN" sim-core spawn_positions_for_seed_42_are_pinned <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        let mut ore_rng = ChaCha8Rng::seed_from_u64(seed ^ STREAM_ORE);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        let mut ore_rng = ChaCha8Rng::seed_from_u64(seed ^ STREAM_WORLDGEN);\n'))
PY

# 3b is the hazard Key decision 1 names: ore drawing from a stream another feature reads after it.
# It SURVIVED against tree_density_for_seed_42_is_deterministic_and_in_target_band (first run,
# 2026-10-11): the trunk count stays in its 230..=300 band. The pinned tree positions see it: 7
# scenario tests, a save_load test and the fingerprint fail; the row names the 12.7 tree pin.
mutation "3b ore draws from the tree stream" sim-core cutting_one_tree_fells_it_whole_and_leaves_the_touching_neighbour_standing <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = ('        let mut ore_rng = ChaCha8Rng::seed_from_u64(seed ^ STREAM_ORE);\n'
       '        worldgen::place_ore(dims, &heights, &mut tiles, &mut ore_rng);\n'
       '        let camp_origin = worldgen::camp_origin(dims, &heights);\n'
       '        let mut tree_rng = ChaCha8Rng::seed_from_u64(seed ^ STREAM_TREES);\n')
assert s.count(old) == 1
p.write_text(s.replace(old, '        let camp_origin = worldgen::camp_origin(dims, &heights);\n'
                            '        let mut tree_rng = ChaCha8Rng::seed_from_u64(seed ^ STREAM_TREES);\n'
                            '        worldgen::place_ore(dims, &heights, &mut tiles, &mut tree_rng);\n'))
PY

mutation "4 ore may replace the surface" sim-core ore_is_interior_never_exposed_and_gems_are_deep_and_rare_over_16_seeds <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/worldgen.rs'); s = p.read_text()
old = 'x + 1 < dims.x && y + 1 < dims.y && z + 2 < height_of(x, y)\n'
assert s.count(old) == 1
s = s.replace(old, 'x + 1 < dims.x && y + 1 < dims.y && z <= height_of(x, y)\n')
old = '                    && tiles[index(dims, x, y, z)] == Tile::Solid(Material::Stone)\n'
assert s.count(old) == 1
s = s.replace(old, '                    && matches!(tiles[index(dims, x, y, z)], Tile::Solid(_))\n')
old = '            if tiles[at] == Tile::Solid(Material::Stone) {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            if matches!(tiles[at], Tile::Solid(_)) {\n'))
PY

mutation "5 gems at any depth" sim-core ore_is_interior_never_exposed_and_gems_are_deep_and_rare_over_16_seeds <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/worldgen.rs'); s = p.read_text()
old = '            height_of(x, y) - z >= GEM_MIN_DEPTH\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            height_of(x, y) - z >= 0\n'))
PY

mutation "6 the channel's ramp loses its ore" sim-core channelling_over_ore_yields_the_ore_and_opens_a_step_with_its_own_ramp <<'PY'
import pathlib
# Re-pointed 2026-10-13 (12.10b): D1 retired 'a channel over ore leaves Ramp(Stone)': the ramp keeps its own material, so the
# sabotage is the ramp losing an ore material.
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    .set_tile(base, Tile::Ramp(material));\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    .set_tile(base, Tile::Ramp(Material::Stone));\n'))
PY

mutation "7 sim_will_keep drops ramps" gui the_dig_preview_keeps_a_ramp_cell <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        DesignateMode::Dig => matches!(mirror.tile(tile), Some(Tile::Solid(_) | Tile::Ramp(_))),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        DesignateMode::Dig => matches!(mirror.tile(tile), Some(Tile::Solid(_))),\n'))
PY

mutation "8 tui bloodstone glyph is stone's" tui every_look_is_pinned <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/palette.rs'); s = p.read_text()
old = "            glyph: '$',\n"
assert s.count(old) == 1
p.write_text(s.replace(old, "            glyph: '█',\n"))
PY

mutation "9 gui bloodstone colour is stone's" gui an_ore_cell_on_the_cut_face_draws_in_its_own_colour <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/appearance.rs'); s = p.read_text()
old = '        Material::Bloodstone => Color::srgb_u8(120, 196, 176),\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        Material::Bloodstone => Color::srgb_u8(60, 70, 92),\n'))
PY

# Review run 1 (AC 4's other half). Only Ice, the loop's LAST material, so a kill also proves the
# test reaches every material rather than stopping at Soil.
mutation "10 ice yields tetrahedrite" sim-core digging_soil_snow_and_ice_still_yields_stone <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = ('        Material::Stone\n'
       '        | Material::Soil\n'
       '        | Material::Ice\n'
       '        | Material::Snow\n')
assert s.count(old) == 1
p.write_text(s.replace(old, '        Material::Ice => ItemKind::Tetrahedrite,\n'
                            '        Material::Stone\n'
                            '        | Material::Soil\n'
                            '        | Material::Snow\n'))
PY

# Review run 1 (Wolf's snow-cap ruling): ore caps like stone again, as the dev's NOTE had it.
mutation "11 ore takes a snow cap" gui a_dug_ore_floor_draws_in_its_own_colour_not_a_snow_cap <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = ('                && material != Material::Tetrahedrite\n'
       '                && material != Material::Bloodstone\n')
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY
