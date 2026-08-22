# Mutation set for story 5.1. Run:
# scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/5-1-the-world-grows-things-that-glow.sh

mutation "tree yield guard always spawns stone" sim-core execute_jobs_digs_tree_materials_without_spawning_items <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        if yields_stone {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        if yields_stone || true {\n'))
PY

mutation "tree placement ignores the camp clearing" sim-core pines_use_both_tree_materials_and_leave_the_camp_clear <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/worldgen.rs'); s = p.read_text()
old = '''            if (x as i32 - camp.x).abs() <= camp_radius + 1
                && (y as i32 - camp.y).abs() <= camp_radius + 1
            {
                continue;
            }
'''
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "trees consume the worldgen stream" sim-core spawn_positions_for_seed_42_are_pinned <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '        let mut tree_rng = ChaCha8Rng::seed_from_u64(seed ^ STREAM_TREES);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        let mut tree_rng = ChaCha8Rng::seed_from_u64(seed ^ STREAM_WORLDGEN);\n'))
PY

mutation "the emitter draw pass is deleted" tui growing_world_instrument_counts_change_with_trees_and_emitters <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
# Re-pointed 2026-08-22: 5.2 moved `tui` off `snapshot.*` onto the shared client-core mirror,
# which rotted this anchor. The seam is unchanged.
old = '''    for entity in mirror.entities() {
        match entity.kind {
            EntityKind::Dwarf => {}
            EntityKind::Torch | EntityKind::Campfire => {
                if let Some(index) = screen_index(entity.pos) {
                    framebuffer.cells[index] = entity_cell(entity.kind, entity.state);
                }
            }
        }
    }

'''
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "delta drops emitters while snapshot keeps them" simd snapshot_and_delta_carry_the_same_emitters <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = '            .chain(world.emitters().into_iter().map(emitter_entity))\n'
assert s.count(old) == 2
at = s.rfind(old)
p.write_text(s[:at] + s[at + len(old):])
PY

mutation "terrain amplitude returns to four" sim-core default_world_has_mountainous_height_span <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/worldgen.rs'); s = p.read_text()
old = '            let height = (dims.z as f64 / 2.0 + (noise * 2.0 - 1.0) * 12.0).round();\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '            let height = (dims.z as f64 / 2.0 + (noise * 2.0 - 1.0) * 4.0).round();\n'))
PY

mutation "dwarf wander homes return to random spawn cells" sim-core idle_dwarves_stay_standable_and_inside_the_camp <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = '                    home: camp,\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                    home: pos,\n'))
PY

mutation "lantern saves reach the live world" simd loading_rejects_static_lantern_emitters_before_the_wire_bridge <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/main.rs'); s = p.read_text()
old = '''            if *light == sim_core::LightKind::Lantern {
                bail!("save emitter {id} uses unsupported lantern kind");
            }
'''
assert s.count(old) == 1
p.write_text(s.replace(old, ''))
PY

mutation "the shipped world seed changes unnoticed" sim-core default_world_has_mountainous_height_span <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = 'pub const DEFAULT_SEED: u64 = 0xF005_7E1A;\n'
assert s.count(old) == 1
p.write_text(s.replace(old, 'pub const DEFAULT_SEED: u64 = 42;\n'))
PY
