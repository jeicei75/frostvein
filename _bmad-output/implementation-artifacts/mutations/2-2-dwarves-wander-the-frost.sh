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
# Re-pointed 2026-08-22: `spawn_dwarves` took `&heights` when this was written and now takes
# `camp_origin`. Intent unchanged -- reuse the worldgen stream (`rng`, still in scope) instead of
# the dedicated spawn stream, which is exactly what the seed pin exists to catch.
old = "        world.spawn_dwarves(camp_origin, &mut spawn_rng);\n"
assert s.count(old) == 1
p.write_text(s.replace(old, "        world.spawn_dwarves(camp_origin, &mut rng);\n"))
PY

# --- Added by code review (2026-08-03). One per review patch: a patch whose test
# --- survives its own sabotage is the exact failure 2.1's review shipped.

# REMOVED 2026-08-22 -- obsolete, not repaired, and verified so by running it: the repaired row
# SURVIVED. `view::initial` no longer reads entity positions; the camera is the world centre
# (`dims.x / 2, dims.y / 2`), so recomputing it per frame is a NO-OP and the defect this row
# described -- re-centring on entity 0 and pinning that dwarf to the middle of the screen --
# structurally cannot happen. NOTE the comment in `tui/src/main.rs` above the frames loop still
# warns about it and is now stale.

# The discriminating case for tightening AC12's assertion: a Walk arm that changes the
# GLYPH while keeping idle's colour. `assert_ne!` on the whole Cell passed this happily.
mutation "walk is marked by a different glyph instead of a different colour" tui walking_and_idle_dwarves_render_different_colors <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/palette.rs'); s = p.read_text()
old = """        (EntityKind::Dwarf, JobState::Walk) => Cell {
            glyph: '☺',
            fg: (214, 154, 78),
        },
"""
assert old in s
p.write_text(s.replace(old, """        (EntityKind::Dwarf, JobState::Walk) => Cell {
            glyph: '☻',
            fg: (150, 112, 62),
        },
"""))
PY

mutation "bridge maps Walk to the Work wire name" simd every_job_state_maps_to_its_named_wire_variant <<'PY'
import pathlib
p = pathlib.Path('crates/simd/src/bridge.rs'); s = p.read_text()
old = "        sim_core::JobState::Walk => protocol::JobState::Walk,\n"
assert old in s
p.write_text(s.replace(old, "        sim_core::JobState::Walk => protocol::JobState::Work,\n"))
PY

# --- Observability-instrument hardening (2026-08-03). The instrument is how this project
# --- evidences "the world visibly lives", so it needs the same sabotage bar as the feature.

mutation "frames capture goes colourless without saying so" tui the_instrument_refuses_to_be_silently_colourless <<'PY'
import pathlib, re
p = pathlib.Path('crates/tui/src/main.rs'); s = p.read_text()
old = re.search(r"    if colour_is_suppressed\(\) \{.*?\n    \}\n", s, re.S)
assert old, "colour warning block not found"
p.write_text(s.replace(old.group(0), ""))
PY

mutation "render drops the entity state and colours every dwarf idle" tui a_walking_dwarf_reaches_the_capture_wearing_the_walk_colour <<'PY'
import pathlib
p = pathlib.Path('crates/tui/src/view.rs'); s = p.read_text()
# Re-pointed 2026-08-22: the dwarf draw moved into a crowd/carrier branch, so the trailing
# semicolon this anchored on is gone.
old = '                entity_cell(entity.kind, entity.state)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '                entity_cell(entity.kind, protocol::JobState::Idle)\n'))
PY
