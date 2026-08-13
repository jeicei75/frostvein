# Mutation set for story 5.3. Run alone: scripts/mutate.sh \
#   _bmad-output/implementation-artifacts/mutations/5-3-a-window-onto-the-valley.sh

mutation "snapshot rebuild is disabled" gui snapshot_rebuild_reaches_reconcile_even_when_changes_are_empty <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    if rebuild_terrain {\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    if false {\n'))
PY

# NOTE: this sabotage must COMPILE. Its first version deleted the `if !solid { return false }`
# guard and left a bare `matches!(...)` expression mid-function, which is a syntax error
# (`expected `;`, found `NEIGHBOURS``). mutate.sh reports any non-zero cargo exit as KILLED, so a
# non-compiling mutation prints no assertion and still reads as a kill — the table claimed five
# kills while pinning four. Found by the orchestrator re-running the table at verification
# (2026-08-13). Keep the neighbour scan's removal, not the guard's: this must change BEHAVIOUR
# (every solid tile reads as exposed, 315,068 cubes instead of 53,365), which is what AC13 pins.
mutation "exposed terrain returns every solid tile" gui exposed_predicate_keeps_boundary_solids_but_hides_fully_enclosed_ones <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = """    NEIGHBOURS.into_iter().any(|delta| {
        let neighbour = [
            position[0] + delta[0],
            position[1] + delta[1],
            position[2] + delta[2],
        ];
        !matches!(mirror.tile(neighbour), Some(Tile::Solid(_)))
    })
"""
assert s.count(old) == 1
p.write_text(s.replace(old, '    true\n'))
PY

mutation "world transform flips handedness" gui coordinate_transform_preserves_the_pinned_handedness <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/transform.rs'); s = p.read_text()
old = '    Vec3::new(x as f32, z as f32, -y as f32)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    Vec3::new(x as f32, z as f32, y as f32)\n'))
PY

mutation "reconciliation ignores the simulation id" gui terrain_ids_never_satisfy_a_simulation_id_lookup <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = 'if let Some((bevy_entity, _)) = projected.iter().find(|(_, marker)| marker.0 == id) {'
assert s.count(old) == 1
p.write_text(s.replace(old, 'if let Some((bevy_entity, _)) = projected.iter().find(|(_, marker)| marker.0 == 0) {'))
PY

mutation "camera pitch clamp is removed" gui orbit_reaches_every_yaw_and_clamps_pitch <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/camera.rs'); s = p.read_text()
old = '        self.pitch = (self.pitch + pitch_delta).clamp(MIN_PITCH, MAX_PITCH);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        self.pitch += pitch_delta;\n'))
PY
