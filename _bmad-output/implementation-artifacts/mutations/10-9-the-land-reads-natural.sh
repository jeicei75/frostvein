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
old_signature = 'pub(crate) fn layered_terrain(dims: Dims, heights: &[u32], _rng: &mut ChaCha8Rng) -> Vec<Tile> {'
assert s.count(old_signature) == 1
old_surface = '''            let surface = surface_material(
                biome_at(dims, x, y),
                local_gradient(dims, heights, x, y),
                x,
                y,
            );'''
assert s.count(old_surface) == 1
new_surface = '''            let surface = if rng.random::<bool>() {
                Material::Snow
            } else {
                Material::Ice
            };'''
s = s.replace(old_signature, old_signature.replace('_rng', 'rng'))
p.write_text(s.replace(old_surface, new_surface))
PY

mutation "constant lake biome is still consumed by the surface rule" sim-core worldgen::tests::biome_decision_is_consumed_by_the_surface_material_rule <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/worldgen.rs'); s = p.read_text()
old = '    if field <= 1.0 {'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if true {'))
PY

mutation "lake basin no longer flattens its ice" sim-core frozen_lake_is_a_flat_contiguous_ice_region_away_from_camp <<'PY'
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

mutation "rebased real-world control rejects a stale quad count" py scripts.tests.test_resolution_bench.ResolutionGeometryTests.test_control_check_requires_the_real_world_literals <<'PY'
import pathlib
p = pathlib.Path('scripts/bench/resolution_bench.py'); s = p.read_text()
old = 'CONTROL_QUADS = 12_132'
assert s.count(old) == 1
p.write_text(s.replace(old, 'CONTROL_QUADS = 12_131'))
PY
