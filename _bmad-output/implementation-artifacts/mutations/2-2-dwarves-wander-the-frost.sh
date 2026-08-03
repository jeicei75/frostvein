# Mutation set for story 2.2. Run:
# scripts/mutate.sh _bmad-output/implementation-artifacts/mutations/2-2-dwarves-wander-the-frost.sh

mutation "wander never assigns the chosen position" sim-core dwarves_spawn_idle_and_wander_in_staggered_id_order <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = "                *pos = candidates[rng.0.random_range(0..n)];\n"
assert old in s
p.write_text(s.replace(old, "                let _ = candidates[rng.0.random_range(0..n)];\n"))
PY

mutation "wander choice is constant candidate zero" sim-core wander_directions_are_not_constant <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = "                *pos = candidates[rng.0.random_range(0..n)];\n"
assert old in s
p.write_text(s.replace(old, "                *pos = candidates[0];\n"))
PY

mutation "wander cooldown never resets" sim-core wander_rest_is_ten_ticks <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = "        wander.cooldown = WANDER_REST_TICKS;\n"
assert old in s
p.write_text(s.replace(old, "        wander.cooldown = 0;\n"))
PY

mutation "standability ignores the supporting tile" sim-core terrain_identifies_standable_tiles <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = """        matches!(self.tile(p), Some(Tile::Empty))
            && matches!(
                self.tile(Pos { z: p.z - 1, ..p }),
                Some(Tile::Solid(_) | Tile::Ramp(_))
            )
"""
assert old in s
p.write_text(s.replace(old, "        matches!(self.tile(p), Some(Tile::Empty))\n"))
PY

mutation "bridge hardcodes every job state to idle" simd every_job_state_maps_to_its_named_wire_variant <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = """    match state {
        sim_core::JobState::Idle => protocol::JobState::Idle,
        sim_core::JobState::Walk => protocol::JobState::Walk,
        sim_core::JobState::Work => protocol::JobState::Work,
    }
"""
assert old in s
p.write_text(s.replace(old, "    let _ = state;\n    protocol::JobState::Idle\n"))
PY

mutation "wander radius widens from three to six" sim-core dwarves_stay_standable_and_near_home <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = "const WANDER_RADIUS: i32 = 3;\n"
assert old in s
p.write_text(s.replace(old, "const WANDER_RADIUS: i32 = 6;\n"))
PY

mutation "spawn consumes the worldgen stream again" sim-core spawn_positions_for_seed_42_are_pinned <<'PY'
import pathlib
p = pathlib.Path('crates/sim-core/src/lib.rs'); s = p.read_text()
old = "        world.spawn_dwarves(&heights, &mut spawn_rng);\n"
assert old in s
p.write_text(s.replace(old, "        world.spawn_dwarves(&heights, &mut rng);\n"))
PY
