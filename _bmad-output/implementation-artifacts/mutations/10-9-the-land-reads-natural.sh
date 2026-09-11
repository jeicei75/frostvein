mutation "per-voxel relief hash breaks the AC2 triangle budget" gui project::tests::material_keyed_relief_keeps_the_reported_triangle_count_well_above_flat_ice <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '''fn detail_depth(plane: i32, u: i32, v: i32, subdiv: i32) -> i32 {
    coherent_detail_depth(plane, u, v, subdiv, 3)
}'''
assert s.count(old) == 1
new = '''fn detail_depth(plane: i32, u: i32, v: i32, subdiv: i32) -> i32 {
    let mut value = DETAIL_SEED
        ^ (plane as u32).wrapping_mul(0x9E37_79B1)
        ^ (u as u32).wrapping_mul(0x85EB_CA77)
        ^ (v as u32).wrapping_mul(0xC2B2_AE3D);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7FEB_352D);
    value ^= value >> 15;
    let offset = (value % 5) as i32 - 2;
    offset.abs().min(subdiv - 1)
}'''
p.write_text(s.replace(old, new))
PY

mutation "snow ice coin flip breaks material coherence" sim-core surface_materials_are_coherent_and_snow_prefers_flat_ground <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/worldgen.rs'); s = p.read_text()
old_signature = 'pub(crate) fn layered_terrain(dims: Dims, heights: &[u32]) -> Vec<Tile> {'
assert s.count(old_signature) == 1
old_surface = '''            let surface = surface_material(
                biome_at(dims, x, y),
                local_gradient(dims, heights, x, y),
                x,
                y,
            );'''
assert s.count(old_surface) == 1
new_surface = '''            let surface = if coin_rng.random::<bool>() {
                Material::Snow
            } else {
                Material::Ice
            };'''
s = s.replace(
    old_signature,
    old_signature
    + '\n    let mut coin_rng = <ChaCha8Rng as rand::SeedableRng>::seed_from_u64(0);',
)
p.write_text(s.replace(old_surface, new_surface))
PY

mutation "constant lake biome is still consumed by the surface rule" sim-core worldgen::tests::biome_decision_is_consumed_by_the_surface_material_rule <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/worldgen.rs'); s = p.read_text()
old = '    if field <= 1.0 {'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if true {'))
PY

mutation "lake basin no longer flattens its ice" sim-core worldgen::tests::lake_post_pass_flattens_its_entire_ice_core <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/worldgen.rs'); s = p.read_text()
old = '                heights[(x + y * dims.x) as usize] = lake_height;'
assert s.count(old) == 1
p.write_text(s.replace(old, '                let _ = lake_height;'))
PY

mutation "ridge band shifts from the far edges to the near edges" sim-core worldgen::tests::ridges_only_change_the_far_edge_footprint_and_lake <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/worldgen.rs'); s = p.read_text()
old = '    x < RIDGE_BAND || y >= dims.y.saturating_sub(RIDGE_BAND)'
assert s.count(old) == 1
p.write_text(s.replace(old, '    x >= dims.x - RIDGE_BAND || y < RIDGE_BAND'))
PY

mutation "height field spacing drifts before its post-pass pin" sim-core worldgen::tests::height_field_for_the_default_seed_stays_pinned_before_post_passes <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/worldgen.rs'); s = p.read_text()
old = 'const NOISE_SPACING: u32 = 32;'
assert s.count(old) == 1
p.write_text(s.replace(old, 'const NOISE_SPACING: u32 = 31;'))
PY

mutation "lake footprint moves over the camp samples" sim-core worldgen::tests::camps_stay_outside_the_lake_for_the_default_seed_and_fifty_more <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/worldgen.rs'); s = p.read_text()
old_x = '    let centre_x = dims.x as f64 * 0.22;'
old_y = '    let centre_y = dims.y as f64 * 0.72;'
assert s.count(old_x) == 1
assert s.count(old_y) == 1
s = s.replace(old_x, '    let centre_x = dims.x as f64 * 0.50;')
p.write_text(s.replace(old_y, '    let centre_y = dims.y as f64 * 0.50;'))
PY

mutation "lake exclusion no longer keeps trees off the ice" sim-core worldgen::tests::trees_do_not_grow_out_of_the_lake <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/worldgen.rs'); s = p.read_text()
old = '''            if biome_at(dims, x, y) == Biome::Lake {
                continue;
            }'''
assert s.count(old) == 1
p.write_text(s.replace(old, '            if false { continue; }'))
PY

mutation "rebased real-world control rejects a stale quad count" py scripts.tests.test_resolution_bench.ResolutionGeometryTests.test_control_check_requires_the_real_world_literals <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/resolution_bench.py'); s = p.read_text()
old = 'CONTROL_QUADS = 12_322'
assert s.count(old) == 1
p.write_text(s.replace(old, 'CONTROL_QUADS = 12_321'))
PY

mutation "offline ice relief stops matching the client's flat ice" py scripts.tests.test_resolution_bench.ResolutionGeometryTests.test_ice_stays_flat_at_subdivision_like_the_client <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/resolution_bench.py'); s = p.read_text()
# Re-pointed at review: the dispatch gained named material sets and an exhaustive-guard raise,
# so the old anchor (a bare trailing `return 0` before `_cell_heights`) no longer exists. Same
# sabotage, same seam -- give the FLAT materials the drifting rule and ice stops being flat.
old = '''    if material in FLAT_MATERIALS:
        return 0'''
assert s.count(old) == 1
new = '''    if material in FLAT_MATERIALS:
        return detail_depth(WORLD_SEED, plane, u, v, k)'''
p.write_text(s.replace(old, new))
PY

mutation "subdiv one instrument reports a zero derived triangle count" gui ingest::tests::subdiv_one_still_spawns_terrain_and_reports_a_derived_triangle_count <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '''pub(crate) fn derived_triangle_count(cubes: usize, snow_caps: usize) -> usize {
    cubes * 12 + snow_caps * 2
}'''
assert s.count(old) == 1
new = '''pub(crate) fn derived_triangle_count(_cubes: usize, _snow_caps: usize) -> usize {
    0
}'''
p.write_text(s.replace(old, new))
PY

mutation "lake ice stops being flat and the pinned AC3 floor moves" gui project::tests::material_keyed_relief_keeps_the_reported_triangle_count_well_above_flat_ice <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '        Material::Ice | Material::TreeTrunk | Material::TreeFoliage => 0,'
assert s.count(old) == 1
new = '''        Material::Ice => coherent_detail_depth(plane, u, v, subdiv, 3),
        Material::TreeTrunk | Material::TreeFoliage => 0,'''
p.write_text(s.replace(old, new))
PY

mutation "python relief dispatch silently flattens an unknown material" py scripts.tests.test_resolution_bench.ResolutionSafetyTests.test_material_relief_refuses_a_material_it_has_no_rule_for <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/resolution_bench.py'); s = p.read_text()
old = '''    raise ValueError(
        f"no relief rule for material {material!r}; the client's match over Material is "
        f"exhaustive, so add the arm on both sides. Known: {sorted(KNOWN_MATERIALS)}"
    )'''
assert s.count(old) == 1
p.write_text(s.replace(old, '    return 0'))
PY

mutation "scoured ground goes back to ice and counterfeits the lake" sim-core frozen_lake_is_a_flat_contiguous_ice_region_away_from_camp <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/worldgen.rs'); s = p.read_text()
old = 'Biome::Snowfield if gradient > 0 && (x / 16 + y / 16).is_multiple_of(5) => Material::Stone,'
assert s.count(old) == 1
new = 'Biome::Snowfield if gradient > 0 && (x / 16 + y / 16).is_multiple_of(5) => Material::Ice,'
p.write_text(s.replace(old, new))
PY
