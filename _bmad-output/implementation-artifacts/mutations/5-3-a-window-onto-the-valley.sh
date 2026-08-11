# Mutation set for story 5.3. Run alone: scripts/mutate.sh \
#   _bmad-output/implementation-artifacts/mutations/5-3-a-window-onto-the-valley.sh

mutation "snapshot rebuild is disabled" gui snapshot_rebuild_projects_terrain_even_when_the_mirror_reports_no_changes <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = 'fn snapshot_needs_full_rebuild(rebuild_terrain: bool) -> bool {\n    rebuild_terrain\n}\n'
assert s.count(old) == 1
p.write_text(s.replace(old, 'fn snapshot_needs_full_rebuild(_rebuild_terrain: bool) -> bool {\n    false\n}\n'))
PY

mutation "exposed terrain returns every solid tile" gui exposed_predicate_keeps_boundary_solids_but_hides_fully_enclosed_ones <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    if !matches!(mirror.tile(position), Some(Tile::Solid(_))) {\n        return false;\n    }\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    matches!(mirror.tile(position), Some(Tile::Solid(_)))\n'))
PY

mutation "world transform flips handedness" gui coordinate_transform_preserves_the_pinned_handedness <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/transform.rs'); s = p.read_text()
old = '    Vec3::new(x as f32, z as f32, -y as f32)\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    Vec3::new(x as f32, z as f32, y as f32)\n'))
PY

mutation "reconciliation ignores the simulation id" gui reconciliation_identity_is_the_simulation_id <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/project.rs'); s = p.read_text()
old = '    marker.0 == id\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '    marker.0 == 0\n'))
PY

mutation "camera pitch clamp is removed" gui orbit_reaches_every_yaw_and_clamps_pitch <<'PY'
import pathlib
p = pathlib.Path('crates/gui/src/camera.rs'); s = p.read_text()
old = '        self.pitch = (self.pitch + pitch_delta).clamp(MIN_PITCH, MAX_PITCH);\n'
assert s.count(old) == 1
p.write_text(s.replace(old, '        self.pitch += pitch_delta;\n'))
PY
